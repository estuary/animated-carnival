import { serve } from "https://deno.land/std@0.131.0/http/server.ts";
import { createClient } from "https://esm.sh/@supabase/supabase-js";
import Handlebars from "https://esm.sh/handlebars";
import jsonpointer from "https://esm.sh/jsonpointer.js";
import { corsHeaders } from "../_shared/cors.ts";
import { supabaseClient } from "../_shared/supabaseClient.ts";

const ENCRYPTION_SERVICE =
  "https://config-encryption.estuary.dev/v1/encrypt-config";

const CREDENTIALS_KEY = "credentials";

export async function encryptConfig(req: Record<string, any>) {
  const { connector_id, config, schema } = req;

  const { data, error } = await supabaseClient
    .from("connectors")
    .select("oauth2_client_id,oauth2_client_secret")
    .eq("id", connector_id)
    .single();

  if (error != null) {
    return new Response(JSON.stringify({ error }), {
      headers: {
        ...corsHeaders,
        "Content-Type": "application/json",
      },
      status: 400,
    });
  }

  const { oauth2_client_id, oauth2_client_secret } = data;

  if (config[CREDENTIALS_KEY]) {
    config[CREDENTIALS_KEY]["client_id"] = oauth2_client_id;
    config[CREDENTIALS_KEY]["client_secret"] = oauth2_client_secret;
  }

  const response = await fetch(ENCRYPTION_SERVICE, {
    method: "POST",
    body: JSON.stringify({ config, schema }),
    headers: {
      accept: "application/json",
      "content-type": "application/json",
    },
  });

  let responseData = JSON.stringify(await response.json());

  // If we can find client_id or client_secret in plaintext in the response,
  // it's not secure to return this response!
  if (
    oauth2_client_id != null &&
    oauth2_client_secret != null &&
    (responseData.includes(oauth2_client_id) ||
      responseData.includes(oauth2_client_secret))
  ) {
    return new Response(
      JSON.stringify({
        error: {
          code: "exposed_secret",
          message: `Request denied: "client id" and "client secret" could have been leaked.`,
          description: `client_id and client_secret were not encrypted as part of this request.
Make sure that they are marked with secret: true in the endpoint spec schema`,
        },
      }),
      {
        headers: { ...corsHeaders, "Content-Type": "application/json" },
        status: response.status,
      }
    );
  }

  return new Response(responseData, {
    headers: { ...corsHeaders, "Content-Type": "application/json" },
    status: response.status,
  });
}
