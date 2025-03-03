<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import { invokeCommand } from '$lib/invoke';

	let id = '';

	async function connect() {
		await invokeCommand('connect', { id });
	}
</script>

<div>
	<InputField bind:value={id} />

	<BigButton on:click={connect}>Connect</BigButton>
</div>
