
create function tests.test_directives()
returns setof text as $$
begin

  -- Replace seed grants with fixtures for this test.
  delete from user_grants;
  insert into user_grants (user_id, object_role, capability) values
    ('11111111-1111-1111-1111-111111111111', 'aliceCo/', 'admin'),
    ('22222222-2222-2222-2222-222222222222', 'bobCo/', 'admin')
  ;
  delete from role_grants;
  insert into role_grants (subject_role, object_role, capability) values
    ('aliceCo/', 'otherCo/', 'write'),
    ('aliceCo/','bobCo/',  'read');

  insert into directives (catalog_prefix, spec, token, single_use) values
    ('aliceCo/', '{"type": "alice"}', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', true),
    ('bobCo/', '{"type": "bob"}',   'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', false);

  -- We're authorized as Alice.
  set role authenticated;
  set request.jwt.claim.sub to '11111111-1111-1111-1111-111111111111';

  -- We see only Alice's directive (which we admin), and not Bob's (despite our read grant).
  return query select results_eq(
    $i$ select catalog_prefix::text, spec::text from directives order by catalog_prefix $i$,
    $i$ values  ('aliceCo/','{"type": "alice"}') $i$,
    'alice directive'
  );

  -- Turn in the Alice directive bearer token to apply it.
  perform exchange_directive_token('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa');

  return query select results_eq(
    $i$
    select d.catalog_prefix::text, d.token::text, a.user_id, a.user_claims::text
    from directives d join applied_directives a on a.directive_id = d.id
    order by d.catalog_prefix;
    $i$,
    $i$ values ('aliceCo/', null, auth.uid(), null) $i$,
    'alice directive is applied and its token is reset'
  );

  -- Turn in the Bob bearer token to apply it.
  perform exchange_directive_token('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb');

  return query select results_eq(
    $i$ select catalog_prefix::text, spec::text from directives order by catalog_prefix $i$,
    $i$ values  ('aliceCo/','{"type": "alice"}'), ('bobCo/','{"type": "bob"}') $i$,
    'bob directive is now visible'
  );

  return query select results_eq(
    $i$
    select d.catalog_prefix::text, d.token::text, a.user_id, a.user_claims::text
    from directives d join applied_directives a on a.directive_id = d.id
    order by d.catalog_prefix;
    $i$,
    $i$
    values ('aliceCo/', null, auth.uid(), null),
      ('bobCo/', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', auth.uid(), null);
    $i$,
    'bob token was not reset (unlike alice it is not single use)'
  );

  -- We can perform an update, but it affects only Alice's directive and not Bob's.
  update directives set catalog_prefix = 'aliceCo/dir/', spec = '{"type": "alice.v2"}';

  return query select results_eq(
    $i$ select catalog_prefix::text, spec::text from directives order by catalog_prefix $i$,
    $i$ values  ('aliceCo/dir/','{"type": "alice.v2"}'), ('bobCo/','{"type": "bob"}') $i$,
    'alice directive is updated and not bob'
  );

  -- We can't change the catalog_prefix to a namespace we don't admin.
  return query select throws_like(
    $i$ update directives set catalog_prefix = 'otherCo/'; $i$,
    'new row violates row-level security policy for table "directives"',
    'attempted change to otherCo fails'
  );

  -- We can update our user claims.
  update applied_directives set user_claims = '{"hi":42}';
  -- And delete them.
  delete from applied_directives;

end
$$ language plpgsql;