<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import Layout from '$lib/layout/layout.svelte';
	import { user } from '$lib/user.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		console.log(user.user);
	});
</script>

<Layout>
	<div class="p-3">
		<div class="p-3">welcome</div>
		<div>
			{#if user.user}
				<Button href="/of/devils/gameday">go to the channel?</Button>
			{:else}
				<Button href="/login">Log in</Button>
			{/if}
		</div>
	</div>
</Layout>
