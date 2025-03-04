<script lang="ts">
	import BigButton from '$lib/components/BigButton.svelte';
	import InputField from '$lib/components/InputField.svelte';
	import { invokeCommand } from '$lib/invoke';

	let loading = false;
	let id = '';

	async function login() {
		loading = true;
		await invokeCommand('login', { provider: 'discord' });
		loading = false;
	}

	async function create() {
		loading = true;
		await invokeCommand('create_sync_profile');
		loading = false;
	}

	async function push() {
		loading = true;
		await invokeCommand('push_sync_profile');
		loading = false;
	}

	async function clone() {
		loading = true;
		await invokeCommand('clone_sync_profile', { id });
		loading = false;
	}

	async function pull() {
		loading = true;
		await invokeCommand('pull_sync_profile');
		loading = false;
	}
</script>

<div class="flex flex-col gap-4">
	<div>
		<BigButton disabled={loading} on:click={login}>Login</BigButton>
	</div>

	<div>
		<BigButton disabled={loading} on:click={create}>Create</BigButton>
		<BigButton disabled={loading} on:click={push}>Push</BigButton>
	</div>

	<div>
		<InputField bind:value={id} />
		<BigButton disabled={loading} on:click={clone}>Clone</BigButton>
		<BigButton disabled={loading} on:click={pull}>Pull</BigButton>
	</div>
</div>
