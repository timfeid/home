<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import '../app.css';
	import Home from '$lib/components/app/home.svg?component';
	import { injectAnalytics } from '@vercel/analytics/sveltekit';
	import { dev } from '$app/environment';
	import { injectSpeedInsights } from '@vercel/speed-insights/sveltekit';
	import { Portal } from 'bits-ui';
	import type { Procedures } from '@feid/bindings';
	import { onMount } from 'svelte';
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

<div class="relative flex min-h-screen w-full flex-col">
	<header
		class="bg-linear-to-r sticky top-0 z-40 flex h-12 items-center border-b border-neutral-100/10 from-neutral-950/50 to-neutral-800/50 backdrop-blur"
	>
		<div class="container mx-auto flex px-4 lg:px-2">
			<Button
				size="sm"
				variant="ghost"
				href="/"
				class="text-base hover:bg-transparent hover:underline"
			>
				<img src="/favicon.png" alt="timfeid.com" class="h-6 w-6" />
				<!-- <Home class="mr-2 h-4 w-4" /> -->
				<div class="ml-2">timfeid.com</div>
			</Button>
			<div class="ml-auto">right</div>
		</div>
	</header>

	<main class="flex w-full flex-grow">
		{@render children()}
	</main>

	<footer class="flex min-h-[3rem] w-full items-center font-mono font-mono text-xs">
		<div class="container mx-auto px-4 text-center">
			&copy; {new Date().getFullYear()} timfeid.com
		</div>
	</footer>

	<div class="pointer-events-none fixed inset-0 -z-10">
		<div
			class="from-background via-background/90 to-background absolute inset-0 bg-gradient-to-b"
		></div>
		<div class="absolute right-0 top-0 h-[500px] w-[500px] bg-zinc-500/10 blur-[100px]"></div>
		<div class="absolute bottom-0 left-0 h-[500px] w-[500px] bg-sky-200/10 blur-[100px]"></div>
	</div>
</div>
