<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import Link from '$lib/components/Link.svelte';

	import Icon from '@iconify/svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';
	import { isChecking, nextUpdate, refreshUpdate } from './Updater.svelte';
	import BigButton from '$lib/components/BigButton.svelte';

	export let open = false;

	let version = '';
	let checkedUpdate = false;

	$: if (open) checkedUpdate = false;

	onMount(async () => {
		version = await getVersion();
	});
</script>

<Popup bind:open title="About">
	<div class="h-3" />
	<img src="logo.png" alt="Logo" class="float-right size-20" />
	<div>
		<h3 class="text-xl font-semibold text-white">Gale</h3>
		<p class="text-primary-300">
			Version {version}
			<br />
			GNU General Public License v3.0
		</p>
		<div class="mt-3 flex items-center gap-2">
			<Icon icon="mdi:file-document" class="text-xl text-white" />
			<Link href="https://github.com/Kesomannen/gale/blob/master/CHANGELOG.md">Changelog</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:file-document" class="text-xl text-white" />
			<Link href="https://github.com/Kesomannen/gale/blob/master/privacy_policy.md"
				>Privacy Policy</Link
			>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:github" class="text-xl text-white" />
			<Link href="https://github.com/Kesomannen/gale">GitHub</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:discord" class="text-xl text-white" />
			<Link href="https://discord.gg/sfuWXRfeTt">Discord</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:heart" class="text-xl text-white" />
			<Link href="https://ko-fi.com/kesomannen">Donate</Link>
		</div>
		<div class="mt-3 flex items-center gap-2">
			<BigButton
				on:click={() => refreshUpdate().then(() => (checkedUpdate = true))}
				disabled={$isChecking}
				color="primary"
				class="mr-2"
			>
				<Icon icon="mdi:refresh" class="mr-2" />
				Check for updates</BigButton
			>

			{#if $isChecking}
				<Icon icon="mdi:loading" class="text-primary-400 animate-spin text-xl" />
				<span class="text-primary-400">Checking for updates...</span>
			{:else if checkedUpdate}
				{#if $nextUpdate === null}
					<Icon icon="mdi:check" class="text-primary-300 text-xl" />
					<span class="text-primary-300">You are running the latest version</span>
				{:else}
					<Icon icon="mdi:arrow-up-circle" class="text-accent-400 inline text-xl" />
					<span class="text-accent-400">New version available: {$nextUpdate?.version}</span>
				{/if}
			{/if}
		</div>
	</div>
</Popup>
