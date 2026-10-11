// TODO: fold this into loader_fields.rs or tags.rs of other v3 testing PR

use common::{
    api_v3::ApiV3,
    environment::{TestEnvironment, with_test_environment},
};

pub mod common;

#[actix_rt::test]
async fn get_games() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = test_env.api;

            let games = api.get_games_deserialized().await;

            assert_eq!(games.len(), 1);
            assert_eq!(games[0].name, "enshrouded");
            assert_eq!(games[0].slug, "enshrouded");
        },
    )
    .await;
}
