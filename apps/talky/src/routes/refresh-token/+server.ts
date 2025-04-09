import { client, wrapResponse } from '$lib/client';

export async function POST(req) {
  const token = req.cookies.get('talky_refresh_token');
  if (token) {
    try {
      const accessToken = wrapResponse(await client.auth_refresh_token.query(token));
      return new Response(accessToken.refresh_token, { status: 200 });
    } catch (e) {
      console.error(e);
    }
  }

  return new Response(null, { status: 401 });
}
