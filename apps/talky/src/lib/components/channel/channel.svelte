<script lang="ts">
	import type { Procedures } from '@feid/bindings';
	import { type Niche } from '../niche/niche.svelte';
	import { onMount } from 'svelte';
	import { client } from '$lib/client';
	import ChatArea from '$lib/chat-area/chat-area.svelte';
	import NewsList from '$lib/news-list/news-list.svelte';

	let { niche, slug }: { niche: Niche; slug: string } = $props();
	let channel = $state<undefined | Procedures['niche_find_by_slug']['output']>();

	onMount(async () => {
		const response = await client.channel_find_by_slug.query(slug);
		if (response.status === 'ok') {
			channel = response.data;
		}
	});
</script>

<!-- niche: {niche.name}<br />
{#if channel}
	channel: {channel.name}
{/if} -->

{#if channel}
	{#if channel.slug === 'gameday'}
		<ChatArea {channel} />
	{:else}
		<NewsList />
	{/if}
{/if}
