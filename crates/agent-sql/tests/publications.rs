use agent_sql::{CatalogType, Id};
use sqlx::{types::Uuid, Connection, Row};

const FIXED_DATABASE_URL: &str = "postgresql://postgres:postgres@localhost:5432/postgres";

#[tokio::test]
async fn test_publication_data_operations() {
    let mut conn = sqlx::postgres::PgConnection::connect(&FIXED_DATABASE_URL)
        .await
        .expect("connect");

    let mut txn = conn.begin().await.unwrap();

    // Fixture: insert live_specs, drafts, and draft_specs fixtures.
    sqlx::query(
        r#"
        with p1 as (
          insert into live_specs (id, catalog_name, spec, spec_type, last_build_id, last_pub_id) values
          ('aa00000000000000', 'aliceCo/First/Thing', '1', 'collection', 'bbbbbbbbbbbbbbbb', 'bbbbbbbbbbbbbbbb'),
          ('bb00000000000000', 'aliceCo/second/Thing', '1', 'collection', 'bbbbbbbbbbbbbbbb', 'bbbbbbbbbbbbbbbb'),
          ('cc00000000000000', 'aliceCo/Test/Fixture', '1', 'test', 'bbbbbbbbbbbbbbbb', 'bbbbbbbbbbbbbbbb'),
          ('ff00000000000000', 'aliceCo/unrelated/Thing', '1', 'collection', 'bbbbbbbbbbbbbbbb', 'bbbbbbbbbbbbbbbb')
        ),
        p11 as (
          -- A "stale" flow of second/Thing reading First/Thing, which we'll remove later.
          insert into live_spec_flows (source_id, target_id, flow_type) values
          ('aa00000000000000', 'bb00000000000000', 'collection')
        ),
        p2 as (
          insert into drafts (id, user_id) values
          ('dddddddddddddddd', '11111111-1111-1111-1111-111111111111')
        ),
        p3 as (
          insert into draft_specs (id, draft_id, catalog_name, spec, spec_type) values
          ('1100000000000000', 'dddddddddddddddd', 'aliceCo/First/Thing', '2', 'collection'), -- Exact match.
          -- Match with different casing, including altered casing of the granted prefix.
          ('2200000000000000', 'dddddddddddddddd', 'AliceCo/Second/Thing', null, null),
          ('3300000000000000', 'dddddddddddddddd', 'aliceCo/New/Thing', '2', 'collection'),
          ('4400000000000000', 'dddddddddddddddd', 'otherCo/Not/AliceCo', '2', 'collection'),
          ('5500000000000000', 'dddddddddddddddd', 'AliceCO/Test/Fixture', '2', 'test')
        ),
        p4 as (
          insert into publications (id, user_id, draft_id) values
          ('eeeeeeeeeeeeeeee', '11111111-1111-1111-1111-111111111111','dddddddddddddddd')
        )
        select 1;
        "#,
    )
    .execute(&mut txn)
    .await
    .unwrap();

    let draft_id = Id::new([0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd, 0xdd]);
    let pub_id = Id::new([0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee]);
    let alice = Uuid::from_bytes([
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11,
    ]);

    // Create new `live_specs` where they don't already exist, with a NULL `spec_type`.
    agent_sql::publications::insert_new_live_specs(draft_id, pub_id, &mut txn)
        .await
        .unwrap();

    // Expect live_specs are created for new specs.
    let flows = sqlx::query(
        "select format('%s: %L', catalog_name, spec_type) from live_specs order by catalog_name",
    )
    .fetch_all(&mut txn)
    .await
    .unwrap();
    insta::assert_json_snapshot!(flows
        .iter()
        .map(|r| -> String { r.get(0) })
        .collect::<Vec<_>>(), @r###"
    [
      "aliceCo/First/Thing: 'collection'",
      "aliceCo/New/Thing: NULL",
      "aliceCo/second/Thing: 'collection'",
      "aliceCo/Test/Fixture: 'test'",
      "aliceCo/unrelated/Thing: 'collection'",
      "otherCo/Not/AliceCo: NULL"
    ]
    "###);

    // Expect we resolve the correct specifications, with corresponding grants.
    let resolved = agent_sql::publications::resolve_spec_rows(draft_id, alice, &mut txn)
        .await
        .unwrap();
    insta::assert_json_snapshot!(resolved, {
      "[].live_spec_id" => "<redacted>",
    });

    // Remove a stale flow of a now-deleted spec.
    agent_sql::publications::delete_stale_flow(
        Id::new([0xbb, 0, 0, 0, 0, 0, 0, 0]),
        CatalogType::Collection,
        &mut txn,
    )
    .await
    .unwrap();

    // Insert a number of flows between `aliceCo/Test/Fixture` and other specs,
    // using catalog names which differ on case. Expect all flows are resolved.
    agent_sql::publications::insert_live_spec_flows(
        Id::new([0xcc, 0, 0, 0, 0, 0, 0, 0]),
        &Some(agent_sql::CatalogType::Test),
        Some(vec!["aliceco/first/thing"]),
        Some(vec!["ALICECO/FIRST/THING", "ALICECO/SECOND/THING"]),
        &mut txn,
    )
    .await
    .unwrap();

    let flows = sqlx::query(
        "select format('%s => %s (%s)', source_id, target_id, flow_type) from live_spec_flows order by source_id, target_id;",
    )
    .fetch_all(&mut txn)
    .await
    .unwrap();

    insta::assert_debug_snapshot!(flows
        .iter()
        .map(|r| -> String { r.get(0) })
        .collect::<Vec<_>>(), @r###"
    [
        "aa:00:00:00:00:00:00:00 => cc:00:00:00:00:00:00:00 (test)",
        "cc:00:00:00:00:00:00:00 => aa:00:00:00:00:00:00:00 (test)",
        "cc:00:00:00:00:00:00:00 => bb:00:00:00:00:00:00:00 (test)",
    ]
    "###);

    // Apply all updates to `live_specs` and delete from `draft_specs`.
    // Also insert into `publication_specs` history table.
    // Note the casing change from `aliceCo/second/Thing` => `aliceCo/Second/Thing`.
    for row in resolved {
        agent_sql::publications::update_published_live_spec(
            &row.catalog_name,
            Some(&"an/image".to_string()),
            Some(&"a-tag".to_string()),
            &row.draft_spec,
            &row.draft_type,
            row.live_spec_id,
            pub_id,
            &Some(vec!["reads/from"]),
            &Some(vec!["writes/to"]),
            &mut txn,
        )
        .await
        .unwrap();

        agent_sql::publications::insert_publication_spec(
            row.live_spec_id,
            pub_id,
            Some(&"the details".to_string()),
            &row.draft_spec,
            &row.draft_type,
            alice,
            &mut txn,
        )
        .await
        .unwrap();

        agent_sql::publications::delete_draft_spec(row.draft_spec_id, &mut txn)
            .await
            .unwrap();
    }

    // Expect `draft_specs` is now empty.
    assert!(sqlx::query("select id from draft_specs")
        .fetch_optional(&mut txn)
        .await
        .unwrap()
        .is_none());

    // Expect `live_specs` reflects our updates.
    let flows = sqlx::query("select to_json(l) from live_specs l order by catalog_name")
        .fetch_all(&mut txn)
        .await
        .unwrap();

    insta::assert_json_snapshot!(flows
        .iter()
        .map(|r| -> serde_json::Value { r.get(0) })
        .collect::<Vec<_>>(),
    {
      "[].id" => "<redacted-id>",
      "[].created_at" => "<redacted-timestamp>",
      "[].updated_at" => "<redacted-timestamp>",
    });
}
