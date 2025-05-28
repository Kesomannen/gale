<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import type { ListedSyncProfile } from '$lib/models';
	import { games, refreshProfiles } from '$lib/stores';
	import { timeSince } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import { profiles as allProfiles } from '$lib/stores';
	import { pushInfoToast } from '$lib/toast';
	import { confirm } from '@tauri-apps/plugin-dialog';

	export let open: boolean;
	export let onClose: () => void;
	export let profiles: ListedSyncProfile[];

	$: sortedProfiles = profiles.toSorted(
		(a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
	);

	async function importProfile(profile: ListedSyncProfile) {
		open = false;
		await invokeCommand('clone_sync_profile', { name: profile.name, id: profile.id });
		await refreshProfiles();
	}

	async function deleteProfile(profile: ListedSyncProfile) {
		let confirmed = await confirm(
			`Are you sure you want to delete ${profile.name} from the database? This will disconnect all subscribers and prevent you from pushing further updates!`
		);
		if (!confirmed) return;

		await invokeCommand('delete_sync_profile', { id: profile.id });

		let index = profiles.indexOf(profile);
		profiles.splice(index, 1);
		// force reactivity
		profiles = profiles;

		await pushInfoToast({ message: 'Deleted sync profile from database.' });
		await refreshProfiles();
	}
</script>

<Popup bind:open {onClose} title="Owned sync profiles">
	<div class="mt-2 flex max-h-80 flex-col overflow-y-auto">
		{#if sortedProfiles.length === 0}
			<div class="text-primary-200 w-full text-center text-lg">No profiles found</div>
		{/if}

		{#each sortedProfiles as profile (profile.id)}
			<div
				class="group text-primary-400 hover:bg-primary-700 flex items-center gap-1 rounded-lg px-4 py-2"
			>
				<div class="mr-auto">
					<div>
						<span class="font-medium text-white">{profile.name}</span>

						<span class="text-primary-300 bg-primary-900 ml-1 rounded px-2 py-0.5 font-mono">
							{profile.id}
						</span>
					</div>

					<div>
						<span>{games.find((game) => game.slug === profile.community)?.name}</span>

						<span class="text-primary-500 mx-1">|</span>

						<span>
							<Icon icon="mdi:clock-outline" class="mb-0.5 inline text-sm" />
							{timeSince(new Date(profile.updatedAt))} ago</span
						>
					</div>
				</div>

				{#if !allProfiles.some((other) => other.sync?.id === profile.id)}
					<Button.Root
						class="text-primary-400 hover:bg-accent-600 hover:text-accent-200  rounded p-1 text-lg"
						on:click={() => importProfile(profile)}
					>
						<Icon icon="mdi:download" />
					</Button.Root>
				{/if}

				<Button.Root
					class="text-primary-400 rounded p-1 text-lg hover:bg-red-600 hover:text-red-200"
					on:click={() => deleteProfile(profile)}
				>
					<Icon icon="mdi:delete" />
				</Button.Root>
			</div>
		{/each}
	</div>
</Popup>
