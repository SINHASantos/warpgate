pub mod api;
use poem::{EndpointExt, IntoEndpoint, Route};
use poem_openapi::OpenApiService;
use warpgate_common::version::warpgate_version;
use warpgate_core::Services;

pub fn admin_api_app(services: &Services) -> impl IntoEndpoint {
    let api_service =
        OpenApiService::new(crate::api::get(), "Warpgate admin API", warpgate_version())
            .server("/@warpgate/admin/api");

    let ui = api_service.stoplight_elements();
    let spec = api_service.spec_endpoint();
    let db = services.db.clone();
    let config = services.config.clone();
    let config_provider = services.config_provider.clone();
    let recordings = services.recordings.clone();
    let state = services.state.clone();

    Route::new()
        .nest("", api_service)
        .nest("/playground", ui)
        .nest("/openapi.json", spec)
        .at(
            "/recordings/:id/cast",
            crate::api::recordings_detail::api_get_recording_cast,
        )
        .at(
            "/recordings/:id/stream",
            crate::api::recordings_detail::api_get_recording_stream,
        )
        .at(
            "/recordings/:id/tcpdump",
            crate::api::recordings_detail::api_get_recording_tcpdump,
        )
        .at(
            "/sessions/changes",
            crate::api::sessions_list::api_get_sessions_changes_stream,
        )
        .data(db)
        .data(config_provider)
        .data(state)
        .data(recordings)
        .data(config)
}
