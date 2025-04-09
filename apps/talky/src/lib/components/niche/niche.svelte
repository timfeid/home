<script lang="ts">
  import { client } from '$lib/client';
  import UserList from '$lib/user-list/user-list.svelte';
  import type { Procedures } from '@feid/bindings';
  import { onMount, type Snippet } from 'svelte';
  import ChannelsSidebar from './channels-sidebar.svelte';

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
  <ChannelsSidebar {niche} />
  <div class="flex flex-1 flex-col">
    {@render withNiche({ niche })}
  </div>
  <div>
    <UserList {niche} />
  </div>
{/if}
