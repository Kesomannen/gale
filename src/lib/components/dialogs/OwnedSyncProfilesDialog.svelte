<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import * as api from '$lib/api';
	import type { ListedSyncProfile } from '$lib/types';
	import { games, refreshProfiles, profiles as allProfiles } from '$lib/stores.svelte';
	import { capitalize, timeSince } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { pushInfoToast } from '$lib/toast';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import IconButton from '$lib/components/ui/IconButton.svelte';

	type Props = {
		open: boolean;
		onClose: () => void;
		profiles: ListedSyncProfile[];
	};

	let { open = $bindable(), onClose, profiles = $bindable() }: Props = $props();

	let sortedProfiles = $derived(
		profiles.toSorted((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
	);

	async function importProfile(profile: ListedSyncProfile) {
		open = false;
		await api.profile.sync.clone(profile.id, profile.name);
		await refreshProfiles();
	}

	async function deleteProfile(profile: ListedSyncProfile) {
		let confirmed = await confirm(
			`Are you sure you want to delete ${profile.name} from the database? This will disconnect all subscribers and prevent you from pushing further updates!`
		);
		if (!confirmed) return;

		await api.profile.sync.deleteProfile(profile.id);

		let index = profiles.indexOf(profile);
		profiles.splice(index, 1);

		await pushInfoToast({ message: 'Deleted sync profile from database.' });
		await refreshProfiles();
	}
</script>

<Dialog bind:open onclose={onClose} title="Owned sync profiles">
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
						<span>{games.list.find((game) => game.slug === profile.community)?.name}</span>

						<span class="text-primary-500 mx-1">|</span>

						<span>
							<Icon icon="mdi:clock-outline" class="mb-0.5 inline text-sm" />
							{capitalize(timeSince(profile.updatedAt))} ago</span
						>
					</div>
				</div>

				{#if !$allProfiles.some((other) => other.sync?.id === profile.id)}
					<IconButton
						label="Import"
						icon="mdi:download"
						color="accent"
						class="text-lg"
						onclick={() => importProfile(profile)}
					/>
				{/if}

				<IconButton
					label="Delete"
					icon="mdi:delete"
					color="red"
					class="text-lg"
					onclick={() => deleteProfile(profile)}
				/>
			</div>
		{/each}
	</div>
</Dialog>
