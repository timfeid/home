<script lang="ts">
	import { page } from '$app/state';
	import ChatArea from '$lib/chat-area/chat-area.svelte';
	import Layout from '$lib/layout/layout.svelte';
	import NewsList from '$lib/news-list/news-list.svelte';
</script>

<Layout>
	{#if page.params.channel === 'news'}
		<NewsList />
	{:else}
		<ChatArea />
	{/if}
</Layout>
