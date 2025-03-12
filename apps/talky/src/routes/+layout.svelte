<script lang="ts">
	import { dev } from '$app/environment';
	import { injectAnalytics } from '@vercel/analytics/sveltekit';
	import { injectSpeedInsights } from '@vercel/speed-insights/sveltekit';
	import { onMount } from 'svelte';
	import '../app.css';
	import { client } from '../lib/client';
	import { user } from '../lib/user.svelte';

	injectSpeedInsights();
	injectAnalytics({ mode: dev ? 'development' : 'production' });

	onMount(async () => {
		const response = await client.auth_login.mutate({ username: 'tim', password: 'wat' });
		if (response.status !== 'ok') {
			return;
		}
		user.accessToken = response.data.access_token;
	});

	let { children } = $props();
</script>

<svelte:head>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
	<link
		href="https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap"
		rel="stylesheet"
	/>
</svelte:head>
<!-- <RspcProvider /> -->

{@render children()}
