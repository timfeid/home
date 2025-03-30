<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import { user } from '../lib/user.svelte';
	import { setupPresence } from '../lib/presence.svelte';

	onMount(() => {
		const presence = setupPresence(user);
		return () => presence.cleanup();
	});

	onMount(async () => {
		await user.login({ username: 'tim', password: 'wat' });
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
