<script lang="ts">
	import { page } from '$app/state';
	import Lobby from '../../../lib/components/app/lobby/lobby.svelte';
	import { user } from '../../../lib/user.svelte';

	let joinCode = page.params.slug;
</script>

{#if user.user}
	<Lobby {joinCode} />
{/if}
