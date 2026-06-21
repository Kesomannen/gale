<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import Link from '$lib/components/ui/Link.svelte';

	import Icon from '@iconify/svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import updates from '$lib/state/update.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open?: boolean;
	};

	let { open = $bindable(false) }: Props = $props();

	let version = $state('');
	let checkedUpdate = $state(false);

	$effect(() => {
		if (open) checkedUpdate = false;
	});

	onMount(async () => {
		version = await getVersion();
	});
</script>

<Dialog bind:open title={m.aboutDialog_title()}>
	<div class="h-3"></div>
	<img src="logo.png" alt="Logo" class="float-right size-20" />
	<div>
		<h3 class="text-xl font-semibold text-white">Gale</h3>
		<p class="text-primary-300">
			{m.aboutDialog_version({ version: version })}
			<br />
			GNU General Public License v3.0
		</p>
		<div class="mt-3 flex items-center gap-2">
			<Icon icon="ph:file-fill" class="text-xl text-white" />
			<Link href="https://github.com/Kesomannen/gale/blob/master/CHANGELOG.md"
				>{m.aboutDialog_changelog()}</Link
			>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="ph:github-logo-fill" class="text-xl text-white" />
			<Link href="https://github.com/Kesomannen/gale">GitHub</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="ph:discord-logo-fill" class="text-xl text-white" />
			<Link href="https://discord.gg/sfuWXRfeTt">Discord</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="ph:heart-fill" class="text-xl text-white" />
			<Link href="https://ko-fi.com/kesomannen">{m.aboutDialog_donate()}</Link>
		</div>
		<div class="mt-3 flex items-center gap-2">
			<Button
				onclick={() => updates.refresh().then(() => (checkedUpdate = true))}
				loading={updates.isChecking}
				color="primary"
				class="mr-2"
				icon="ph:arrow-clockwise-fill"
			>
				{m.aboutDialog_checkUpdate()}
			</Button>

			{#if !updates.isChecking && checkedUpdate}
				{#if updates.next}
					<Icon icon="ph:arrow-circle-up-fill" class="text-accent-400 inline text-xl" />
					<span class="text-accent-400"
						>{m.aboutDialog_newVersion({ version: updates.next.version })}</span
					>
				{:else}
					<Icon icon="ph:check-fill" class="text-primary-300 text-xl" />
					<span class="text-primary-300">{m.aboutDialog_latestVersion()}</span>
				{/if}
			{/if}
		</div>
	</div>
</Dialog>
