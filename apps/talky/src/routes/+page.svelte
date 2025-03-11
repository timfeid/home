<script lang="ts">
	import { onMount } from 'svelte';
	import { client } from '../lib/client';

	onMount(async () => {
		const response = await client.mutation(['auth_login', { username: 'tim', password: 'wat' }]);
		console.log(response);
	});
</script>

hello world.
