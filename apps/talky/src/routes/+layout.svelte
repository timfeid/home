<script lang="ts">
	import { ChannelList } from '$lib/channel-list.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Presence } from '$lib/presence.svelte';
	import { user } from '$lib/user.svelte';
	import { onDestroy, onMount, setContext } from 'svelte';
	import '../app.css';
	let { children } = $props();

	const channelList = new ChannelList();
	setContext('channelList', channelList);
	onDestroy(() => channelList.cleanup());

	const presence = new Presence();
	setContext('presence', presence);
	onDestroy(() => presence.cleanup());

	onMount(async () => {
		presence.setup();
		user.setup();
	});
</script>

<Sidebar.Provider>
	{@render children()}
</Sidebar.Provider>
