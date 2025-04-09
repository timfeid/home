import { PUBLIC_API_URL } from '$env/static/public';
import type { Procedures, ProceduresLegacy } from '@feid/bindings';
import { createClient, type Procedure, type ProcedureResult } from './rspc';
import { fetchExecute } from './rspc/UntypedClient';
import { user } from './user.svelte';
import { browser } from '$app/environment';

import { createClient as legacyClient, WebsocketTransport } from '@rspc/client';

export const client = createClient<Procedures>((args) => {
  return fetchExecute(
    {
      url: PUBLIC_API_URL,
      accessToken: user.accessToken,
    },
    args,
  );
});

export const websocketClient = !browser
  ? client
  : legacyClient<ProceduresLegacy>({
      transport: new WebsocketTransport(PUBLIC_API_URL.replace('http', 'ws') + '/ws'),
    });

export function wrapResponse<P extends Procedure>(result: ProcedureResult<P>) {
  if (result.status !== 'ok') {
    throw new Error(JSON.stringify(result.error));
  }

  return result.data;
}
