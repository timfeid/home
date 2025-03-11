<script lang="ts">
	import { page } from '$app/state';
	import Lobby from '../../../lib/components/app/lobby/lobby.svelte';

	let joinCode = page.params.slug;
</script>

<Lobby {joinCode} />
