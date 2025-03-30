import { decodeJwt } from 'jose';
import { client } from './client';

export class UserDetails {
	accessToken = $state<string | undefined>();

	user = $derived.by(() => {
		if (!this.accessToken) {
			return undefined;
		}

		return decodeJwt<{ sub: string }>(this.accessToken);
	});

	async login(details: { username: string; password: string }) {
		const response = await client.auth_login.mutate(details);
		if (response.status !== 'ok') {
			return;
		}
		user.accessToken = response.data.access_token;
	}
}

export const user = new UserDetails();
