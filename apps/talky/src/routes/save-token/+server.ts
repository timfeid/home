export async function POST(req) {
	const { refresh_token } = await req.request.json();
	req.cookies.set('talky_refresh_token', refresh_token, { path: '/' });

	return new Response(null, { status: 201 });
}
