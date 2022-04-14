
-- This script is used to setup replication for the tables
-- the UI uses subscriptions for. Without this the UI will
-- never be sent messages about changes.
--
-- To run: run inside the SQL Editor

begin;

-- Drop the current realtime replication
drop publication if exists supabase_realtime; 

-- Create the realtime replication
create publication supabase_realtime;  

-- Setup the tables the UI uses subscriptions with
alter publication supabase_realtime add table discovers;
alter publication supabase_realtime add table publications;
  
commit;