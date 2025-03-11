import { browser } from '$app/environment';
import { PUBLIC_API_URL } from '$env/static/public';
import type { Procedures } from '@feid/bindings';
// import { FetchTransport, WebsocketTransport, createClient } from '@rspc/client';
import { user } from './user.svelte';
import { createClient, FetchTransport, WebsocketTransport } from '@rspc/client';

const transport = new FetchTransport(PUBLIC_API_URL, async (input, init) => {
	// const refreshing = input.toString().includes('refresh_token');

	return fetch(input, {
		...init,
		headers: {
			authorization: user.accessToken ? `Bearer ${user.accessToken}` : '',
		},
	});
});

const wtransport = browser
	? new WebsocketTransport(PUBLIC_API_URL.replace('http', 'ws') + '/ws')
	: transport;
console.log(wtransport);
export const websocketClient = createClient<Procedures>({
	transport: wtransport,
});

export const client = createClient<Procedures>({
	transport,
});
