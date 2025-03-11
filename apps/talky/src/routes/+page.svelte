<script lang="ts">
	import { onMount } from 'svelte';
	import { client } from '../lib/client';
	import { user } from '../lib/user.svelte';
	import Button from '../lib/components/ui/button/button.svelte';
	import { goto } from '$app/navigation';

	async function doSomething(e: never) {
		const response = await client.lobby_create.mutate();
		if (response.status !== 'ok') {
			return;
		}

		goto('/lobby/' + response.data.join_code);
		console.log(response);
	}
</script>

hello world.

{JSON.stringify(user.user)}

<Button onclick={doSomething}>do something</Button>
