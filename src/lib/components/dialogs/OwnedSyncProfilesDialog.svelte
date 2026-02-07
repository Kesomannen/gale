<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import * as api from '$lib/api';
	import type { ListedSyncProfile } from '$lib/types';
	import { capitalize, timeSince } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { pushInfoToast } from '$lib/toast';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import games from '$lib/state/game.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open: boolean;
		onClose: () => void;
		profiles: ListedSyncProfile[];
	};

	let { open = $bindable(), onClose, profiles: syncProfiles = $bindable() }: Props = $props();

	let sortedProfiles = $derived(
		syncProfiles.toSorted(
			(a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
		)
	);

	async function importProfile(profile: ListedSyncProfile) {
		await games.setActive(profile.community);

		open = false;

		await api.profile.sync.clone(profile.id, profile.name);
	}

	async function deleteProfile(profile: ListedSyncProfile) {
		let confirmed = await confirm(
			m.ownedSyncProfilesDialog_deleteProfile_confirm({ name: profile.name })
		);
		if (!confirmed) return;

		await api.profile.sync.deleteProfile(profile.id);

		let index = syncProfiles.indexOf(profile);
		syncProfiles.splice(index, 1);

		pushInfoToast({ message: m.ownedSyncProfilesDialog_deleteProfile_message() });
	}
</script>

<Dialog bind:open onclose={onClose} title={m.ownedSyncProfilesDialog_title()}>
	<div class="mt-4 flex max-h-80 flex-col space-y-4 overflow-y-auto px-2">
		{#if sortedProfiles.length === 0}
			<div class="text-primary-200 w-full text-center text-lg">
				{m.ownedSyncProfilesDialog_content_1()}
			</div>
		{/if}

		{#each sortedProfiles as profile (profile.id)}
			<div class="group text-primary-400 flex items-center gap-1 rounded-lg">
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
							{m.ownedSyncProfilesDialog_content_2({
								time: capitalize(timeSince(profile.updatedAt))
							})}</span
						>
					</div>
				</div>

				{#if !profiles.list.some((other) => other.sync?.id === profile.id)}
					<IconButton
						label={m.ownedSyncProfilesDialog_button_import()}
						icon="mdi:download"
						color="accent"
						onclick={() => importProfile(profile)}
					/>
				{/if}

				<IconButton
					label={m.ownedSyncProfilesDialog_button_delete()}
					icon="mdi:delete"
					color="red"
					onclick={() => deleteProfile(profile)}
				/>
			</div>
		{/each}
	</div>
</Dialog>
