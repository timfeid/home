<script lang="ts">
	import { client } from '$lib/client';
	import type { Procedures } from '@feid/bindings';
	import { onMount, type Snippet } from 'svelte';

	export type Niche = Procedures['niche_find_by_slug']['output'];
	let { slug, withNiche }: { slug: string; withNiche: Snippet<[{ niche: Niche }]> } = $props();

	onMount(async () => {
		const response = await client.niche_find_by_slug.query(slug);
		if (response.status === 'ok') {
			niche = response.data;
		}
	});

	let niche: undefined | Niche = $state();
</script>

{#if niche}
	{@render withNiche({ niche })}
{/if}
