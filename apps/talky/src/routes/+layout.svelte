<script lang="ts">
	import { setupPresence } from '$lib/presence.svelte';
	import { user } from '$lib/user.svelte';
	import { onMount } from 'svelte';
	import '../app.css';
	let { children } = $props();

	onMount(() => {
		const presence = setupPresence(user);
		return () => presence.cleanup();
	});

	onMount(async () => {
		await user.login({ username: 'tim', password: 'wat' });
	});
</script>

{@render children()}
