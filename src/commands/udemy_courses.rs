use std::time::Instant;

use crate::platforms::udemy::api::UdemyCourse;
use crate::state::UdemyCoursesCache;

const COURSES_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(10 * 60);

#[allow(dead_code)]
async fn get_portal(plugin: &crate::CoursesPlugin) -> String {
    let guard = plugin.udemy_session.lock().await;
    guard
        .as_ref()
        .map(|s| s.portal_name.clone())
        .unwrap_or_else(|| "www".into())
}

fn parse_courses_from_results(results: &[serde_json::Value]) -> Vec<UdemyCourse> {
    results
        .iter()
        .filter_map(|item| {
            let id = item.get("id")?.as_u64()?;
            let title = item.get("title")?.as_str().unwrap_or("").to_string();
            let published_title = item
                .get("published_title")?
                .as_str()
                .unwrap_or("")
                .to_string();
            let url = item
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let image_url = item
                .get("image_240x135")
                .or_else(|| item.get("image_480x270"))
                .or_else(|| item.get("image_url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let num_published_lectures = item
                .get("num_published_lectures")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32);
            let locale = crate::platforms::udemy::api::extract_course_locale(item);

            Some(UdemyCourse {
                id,
                title,
                published_title,
                url,
                image_url,
                num_published_lectures,
                locale,
            })
        })
        .collect()
}

/// Walk a Udemy api-2.0 paginated list endpoint to completion.
///
/// Udemy caps `page_size` at 100 per request regardless of what you ask for, so
/// a single call only ever returns the first 100 items. Each response carries a
/// `next` cursor (an absolute URL) which we follow until the API stops handing
/// one back, accumulating every page's `results`.
async fn fetch_all_pages(
    client: &reqwest::Client,
    first_url: String,
) -> Result<Vec<serde_json::Value>, String> {
    // Safety stop: ~10k items at 100/page. Prevents an infinite loop if the API
    // ever returns a self-referential or malformed `next`.
    const MAX_PAGES: u32 = 100;

    let mut all: Vec<serde_json::Value> = Vec::new();
    let mut next_url = Some(first_url);
    let mut page = 0u32;

    while let Some(url) = next_url {
        page += 1;
        if page > MAX_PAGES {
            tracing::warn!("[udemy-api] hit MAX_PAGES={} cap, stopping pagination", MAX_PAGES);
            break;
        }
        // Be gentle on the API between pages (matches the existing helper).
        if page > 1 {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(
                "[udemy-api] page {} failed: status={}, body={}",
                page,
                status,
                &body[..body.len().min(500)]
            );
            // If the very first page fails there's nothing to show — surface the
            // error. If a later page fails, keep what we already gathered.
            if all.is_empty() {
                return Err(format!("API returned status {}", status));
            }
            break;
        }

        let body = resp.text().await.map_err(|e| format!("Read body failed: {}", e))?;
        let data: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("JSON parse error: {}", e))?;

        if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
            all.extend(results.iter().cloned());
        }

        next_url = data
            .get("next")
            .and_then(|n| n.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
    }

    tracing::info!("[udemy-api] fetched {} items across {} page(s)", all.len(), page);
    Ok(all)
}

async fn fetch_courses_via_api(
    plugin: &crate::CoursesPlugin,
) -> Result<Vec<UdemyCourse>, String> {
    let (client, portal) = {
        let guard = plugin.udemy_session.lock().await;
        let session = guard.as_ref().ok_or("not_authenticated")?;
        (session.client.clone(), session.portal_name.clone())
    };

    let token_len = {
        let guard = plugin.udemy_session.lock().await;
        guard.as_ref().map(|s| s.access_token.len()).unwrap_or(0)
    };
    tracing::info!("[udemy-api] fetching courses: portal={}, token_len={}", portal, token_len);

    // Subscribed (purchased) courses. page_size=100 is Udemy's per-request max;
    // fetch_all_pages follows the `next` cursor to gather every enrollment.
    let subscribed_url = format!(
        "https://{}.udemy.com/api-2.0/users/me/subscribed-courses?fields[course]=id,url,title,published_title,image_240x135,num_published_lectures,locale&ordering=-last_accessed,-access_time&page=1&page_size=100",
        portal
    );
    let subscribed = fetch_all_pages(&client, subscribed_url).await?;
    let mut courses = parse_courses_from_results(&subscribed);

    // Subscription-plan enrollments (Personal Plan, etc.) — also paginated.
    let sub_url = format!(
        "https://{}.udemy.com/api-2.0/users/me/subscription-course-enrollments?fields[course]=id,title,published_title,image_240x135,num_published_lectures,locale&page=1&page_size=100",
        portal
    );
    match fetch_all_pages(&client, sub_url).await {
        Ok(sub_results) => {
            let existing_ids: std::collections::HashSet<u64> =
                courses.iter().map(|c| c.id).collect();
            for c in parse_courses_from_results(&sub_results) {
                if !existing_ids.contains(&c.id) {
                    courses.push(c);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "[udemy-api] subscription enrollments failed (continuing with subscribed): {}",
                e
            );
        }
    }

    tracing::info!("[udemy-api] found {} total courses via direct API", courses.len());

    let mut cache = plugin.udemy_courses_cache.lock().await;
    *cache = Some(UdemyCoursesCache {
        courses: courses.clone(),
        fetched_at: Instant::now(),
    });

    Ok(courses)
}

async fn fetch_courses(
    plugin: &crate::CoursesPlugin,
) -> Result<Vec<UdemyCourse>, String> {
    fetch_courses_via_api(plugin).await
}


pub async fn udemy_list_courses(
    plugin: &crate::CoursesPlugin,
) -> Result<Vec<UdemyCourse>, String> {
    {
        let cache = plugin.udemy_courses_cache.lock().await;
        if let Some(ref cached) = *cache {
            if cached.fetched_at.elapsed() < COURSES_CACHE_TTL {
                return Ok(cached.courses.clone());
            }
        }
    }

    fetch_courses(plugin).await
}


pub async fn udemy_refresh_courses(
    plugin: &crate::CoursesPlugin,
) -> Result<Vec<UdemyCourse>, String> {
    {
        let mut cache = plugin.udemy_courses_cache.lock().await;
        *cache = None;
    }
    fetch_courses(plugin).await
}
