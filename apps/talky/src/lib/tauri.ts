import { invoke, isTauri as forTauri } from '@tauri-apps/api/core';

import { client } from './client';
import { browser } from '$app/environment';
import { type UnlistenFn } from '@tauri-apps/api/event';

export const isTauri = browser && forTauri();

export async function getRefreshTokenFromTauri() {
	return (await invoke('get_refresh_token')) as string | null;
}

export async function saveRefreshTokenTauri(token: string) {
	const response = await invoke('set_refresh_token', { token });

	return response;
}

export async function getAccessTokenWithTauri() {
	const refreshToken = await getRefreshTokenFromTauri();
	if (refreshToken) {
		const response = await client.auth_refresh_token.query(refreshToken);
		if (response.status === 'ok') {
			return response.data.access_token;
		}
	}
}

export async function pushToTalkStart() {
	return await invoke('toggle_push_to_talk', { active: true });
}

export async function pushToTalkEnd() {
	return await invoke('toggle_push_to_talk', { active: false });
}

export function createTauriListeners() {
	if (!isTauri) {
		return () => {};
	}
	console.log('setup listeners.');

	const unsubscribers: UnlistenFn[] = [];
	// listen<string[]>('qr_results', async (results) => {
	// 	// const accountDetails = await client.query(['account.preview', results.payload]);
	// 	// accountDetails.map(createAccount);
	// }).then((unsub) => unsubscribers.push(unsub));

	return () => {
		for (const unsub of unsubscribers) {
			unsub();
		}
	};
}
