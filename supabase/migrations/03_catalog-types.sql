create domain catalog_name as text
  constraint "Must be a valid catalog name"
  check (value ~ '^([[:alpha:][:digit:]\-_.]+/)+[[:alpha:][:digit:]\-_.]+$' and value is nfkc normalized);
comment on domain catalog_name is '
catalog_name is a name within the Flow catalog namespace.

Catalog names consist of Unicode-normalized (NFKC) letters, numbers,
"-", "_", and ".", with components separated by "/" and not ending in "/".

For example: "acmeCo/anvils" or "acmeCo/products/TnT_v4",
but not "acmeCo//anvils/" or "acmeCo/some anvils".
';

create domain catalog_prefix as text
  constraint "Must be a valid catalog prefix"
  check (value ~ '^([[:alpha:][:digit:]\-_.]+/)+$' and value is nfkc normalized);
comment on domain catalog_prefix is '
catalog_name is a prefix within the Flow catalog namespace.

Catalog prefixes consist of Unicode-normalized (NFKC) letters, numbers,
"-", "_", and ".", with components separated by "/" and ending in a final "/".

For example: "acmeCo/anvils/" or "acmeCo/products/TnT_v4/",
but not "acmeCo/anvils" or "acmeCo/some anvils".
';

create type catalog_spec_type as enum (
  -- These correspond 1:1 with top-level maps of models::Catalog.
  'capture',
  'collection',
  'materialization',
  'test'
);

comment on type catalog_spec_type is '
Enumeration of Flow catalog specification types:
"capture", "collection", "materialization", or "test"
';

create function internal.istarts_with(string text, prefix text)
returns boolean as $$
  select starts_with(lower(string), lower(prefix))
$$ language sql immutable;