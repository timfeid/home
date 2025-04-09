import { decodeJwt } from 'jose';
import { client } from './client';
import { isTauri, getAccessTokenWithTauri, saveRefreshTokenTauri } from './tauri/tauri';
import type { Procedures } from '@feid/bindings';

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
      throw new Error(JSON.stringify(response.error));
    }
    user.accessToken = response.data.access_token;

    this.saveTokens(response.data);

    return this;
  }

  private async saveTokens(details: Procedures['auth_refresh_token']['output']) {
    if (isTauri) {
      return saveRefreshTokenTauri(details.refresh_token);
    }

    await fetch(`/save-token`, {
      method: 'post',
      body: JSON.stringify(details),
      headers: {
        'Content-Type': 'application/json',
      },
    });
  }

  public async setup() {
    const token = await this.getFreshAccessToken();
    if (token) {
      this.accessToken = token;
    }
  }

  private async getFreshAccessToken() {
    if (isTauri) {
      return getAccessTokenWithTauri();
    }

    const response = await fetch('/refresh-token', { method: 'post' });
    const token = ((await response.text()) || '').trim();

    return token;
  }
}

export const user = new UserDetails();
