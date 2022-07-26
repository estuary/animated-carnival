
create function tests.test_live_spec_name_constraints()
returns setof text as $$
declare
  test_case record;
begin

  -- Establish initial fixtures.
  insert into live_specs (catalog_name, last_build_id, last_pub_id) values
    ('Some/Name', default, default),
    ('The/Other/Name', default, default),
    ('A/Very/Long/Name', default, default)
    ;

  for test_case in
    select * from (values
      ('some/name', true, null), -- Disallowed name conflict.
      ('THE/OTHER/NAME', true, null), -- Disallowed name conflict.
      ('some/nameName', false, null), -- Allowed prefix.
      ('thethe/Other/Name', false, null), -- Allowed prefix.
      ('Some/Name/Extra', false, 'Some/Name'), -- Is a suffix of existing name.
      ('the/other/Name/suffix', false, 'The/Other/Name'), -- Is a (caseless) suffix of existing name.
      ('A/Very/Long', false, 'A/Very/Long/Name'), -- Is a prefix of existing name.
      ('The/OTHER', false, 'The/Other/Name') -- Is a (caseless) prefix of existing name.
    ) as t("name", "is_duplicate", "has_prefix")
  loop
    case
      when test_case.is_duplicate then
        return query select throws_ok(
          format($i$
            insert into live_specs (catalog_name, last_build_id, last_pub_id) values
              ('%s', default, default);
          $i$, test_case.name),
          23505, -- unique_violation
          'duplicate key value violates unique constraint "idx_live_specs_catalog_name"'
        );
      when test_case.has_prefix is not null then
        return query select throws_ok(
          format($i$
            insert into live_specs (catalog_name, last_build_id, last_pub_id) values
              ('%s', default, default);
          $i$, test_case.name),
          23505, -- unique_violation
          format(
            'Conflicting catalog names: %s vs %s',
            test_case.name, test_case.has_prefix
          )
        );
      else
        insert into live_specs (catalog_name, last_build_id, last_pub_id) values
          (test_case.name, default, default);
    end case;
  end loop;

end;
$$ language plpgsql;

create function tests.test_draft_spec_name_constraints()
returns setof text as $$
declare
  test_case record;
  draft_id  flowid = internal.id_generator();
  draft_id2 flowid = internal.id_generator();
begin

  -- Establish initial fixtures.
  insert into drafts (id, user_id) values
    (draft_id, '11111111-1111-1111-1111-111111111111'),
    (draft_id2, '11111111-1111-1111-1111-111111111111');

  -- Allowed variations which don't conflict on case.
  insert into draft_specs (draft_id, catalog_name) values
    (draft_id, 'Some/Name'),
    (draft_id, 'some/names'),
    (draft_id, 'The/Other/Name'),
    (draft_id, 'theE/other/name'),
    (draft_id, 'the/other/name/suffix'), -- We don't enforce prefix checks with drafts.
    (draft_id, 'a/fixture'),
    (draft_id2, 'The/Other/Name') -- Duplicate name in a different draft.
    ;

  return query select throws_ok(
    $i$
      update draft_specs set catalog_name = 'the/other/name' where catalog_name = 'a/fixture';
    $i$,
    23505, -- unique_violation
    'duplicate key value violates unique constraint "idx_draft_specs_catalog_name"'
  );

end;
$$ language plpgsql;