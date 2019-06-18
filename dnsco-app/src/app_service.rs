use askama::Template;
use std::sync::{Arc, Mutex, MutexGuard};

use dnsco_data::{repos, Database, EventsRepo, StravaApi};
use repos::activities_repo;
use strava;

use crate::activities::ListTemplate;
use crate::templates::index_template;
use crate::{config, AppError};

pub struct Service {
    db: Arc<Database>,
    events_repo: EventsRepo,
    strava_api: Arc<Mutex<StravaApi>>,
    pub urls: config::SiteUrls,
}

impl Service {
    pub fn new(
        db: Arc<Database>,
        strava_api: Arc<Mutex<StravaApi>>,
        urls: config::SiteUrls,
    ) -> Self {
        Self {
            db,
            events_repo: EventsRepo {},
            strava_api,
            urls,
        }
    }

    pub fn hello_world(&self) -> impl Template + '_ {
        let events = self.events_repo.events();
        index_template::new(events, &self.urls)
    }

    pub fn activities(&self) -> Result<ListTemplate, AppError> {
        let connection = self.db.get_connection();

        let repo = activities_repo::Repo {
            connection: &connection,
        };

        Ok(ListTemplate::new(repo.all(), self.urls.update_activities()))
    }

    pub fn update_activities(&self) -> Result<(), strava::Error> {
        let connection = self.db.get_connection();

        let repo = activities_repo::Repo {
            connection: &connection,
        };

        let strava = self.get_strava_api().api()?.activities()?;
        Ok(repo.batch_upsert_from_strava(strava))
    }

    fn get_strava_api(&self) -> MutexGuard<StravaApi> {
        self.strava_api.lock().unwrap()
    }

    pub fn update_oauth_token(
        &self,
        oauth_resp: &strava::oauth::RedirectQuery,
    ) -> Result<strava::oauth::AccessTokenResponse, strava::Error> {
        let mut strava = self.strava_api.lock().unwrap();
        let resp = strava.parsed_oauth_response(&oauth_resp)?;
        strava.set_tokens(&resp);
        Ok(resp)
    }
}
