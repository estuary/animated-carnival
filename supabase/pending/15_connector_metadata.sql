DROP VIEW IF EXISTS live_specs_ext CASCADE;

-- ALTER TABLE IF EXISTS public.connectors
--     DROP COLUMN IF EXISTS logo_url,
--     DROP COLUMN IF EXISTS title,
--     DROP COLUMN IF EXISTS short_description,
--     DROP COLUMN IF EXISTS recommended;

ALTER TABLE IF EXISTS public.connectors
    ADD COLUMN logo_url json_obj,
    ADD COLUMN title json_obj,
    ADD COLUMN short_description json_obj,
    ADD COLUMN recommended boolean;

UPDATE public.connectors SET
    logo_url=json_build_object('en-US',open_graph->'en-US'->>'image'),
    title=json_build_object('en-US',open_graph->'en-US'->>'title'),
    short_description=json_build_object('en-US',detail);

COMMENT ON COLUMN public.connectors.logo_url
    IS 'The url for this connector''s logo image. Represented as a json object with IETF language tags as keys (https://en.wikipedia.org/wiki/IETF_language_tag), and urls as values';

COMMENT ON COLUMN public.connectors.title
    IS 'The title of this connector. Represented as a json object with IETF language tags as keys (https://en.wikipedia.org/wiki/IETF_language_tag), and the title string as values';

COMMENT ON COLUMN public.connectors.short_description
    IS 'A short description of this connector, at most a few sentences. Represented as a json object with IETF language tags as keys (https://en.wikipedia.org/wiki/IETF_language_tag), and the description string as values';

grant select(logo_url, title, short_description, recommended) on table connectors to authenticated;

-- Copied from `10_spec_ext.sql`, except where indicated --

-- Extended view of live catalog specifications.
CREATE VIEW live_specs_ext as
select
  l.*,
  c.external_url as connector_external_url,
  c.id as connector_id,
  -- c.open_graph as connector_open_graph, -- Removed
  c.title as connector_title, -- Added
  c.short_description as connector_short_description, -- Added
  c.logo_url as connector_logo_url, -- Added
  c.recommended as connector_recommended, -- Added
  t.id as connector_tag_id,
  t.documentation_url as connector_tag_documentation_url,
  p.detail as last_pub_detail,
  p.user_id as last_pub_user_id,
  u.avatar_url as last_pub_user_avatar_url,
  u.email as last_pub_user_email,
  u.full_name as last_pub_user_full_name
from live_specs l
left outer join publication_specs p on l.id = p.live_spec_id and l.last_pub_id = p.pub_id
left outer join connectors c on c.image_name = l.connector_image_name
left outer join connector_tags t on c.id = t.connector_id and l.connector_image_tag = t.image_tag,
lateral view_user_profile(p.user_id) u
;
alter view live_specs_ext owner to authenticated;

comment on view live_specs_ext is
  'View of `live_specs` extended with metadata of its last publication';

-- Extended view of user draft specifications.
create view draft_specs_ext as
select
  d.*,
  l.last_pub_detail,
  l.last_pub_id,
  l.last_pub_user_id,
  l.last_pub_user_avatar_url,
  l.last_pub_user_email,
  l.last_pub_user_full_name,
  l.spec as live_spec,
  l.spec_type as live_spec_type
from draft_specs d
left outer join live_specs_ext l
  on d.catalog_name = l.catalog_name
;
alter view draft_specs_ext owner to authenticated;

comment on view draft_specs_ext is
  'View of `draft_specs` extended with metadata of its live specification';

-- End copy from `10_spec_ext.sql` -- 