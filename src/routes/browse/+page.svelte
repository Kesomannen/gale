<script lang="ts">
	import * as api from '$lib/api';
	import {
		type SortBy,
		type Mod,
		type ModId,
		Backend,
		type ModContextItem,
		type DeduplicatedMod
	} from '$lib/types';

	import ModList from '$lib/components/mod-list/ModList.svelte';

	import { onMount, untrack } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import ModListItem from '$lib/components/mod-list/ModListItem.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ModDetails from '$lib/components/mod-list/ModDetails.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import { defaultContextItems } from '$lib/context';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { modQuery } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { pushInfoToast } from '$lib/toast';
	import HelpCard from '$lib/components/ui/HelpCard.svelte';
	import ForeignDownloadDialog from '$lib/components/dialogs/ForeignDownloadDialog.svelte';
	import { shouldWarnForeignDownload } from '$lib/util';
	import DeduplicatedModDetails from '$lib/components/mod-list/DeduplicatedModDetails.svelte';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];
	const contextItems: ModContextItem[] = [
		{
			label: m.browse_contextItem_hideMod(),
			icon: 'mdi:eye-off',
			onclick: async (mod: Mod) => {
				await api.profile.toggleHiddenMod(mod.uuid);
				await refresh();
				pushInfoToast({
					message: m.browse_contextitem_hideMod_message({ name: mod.name })
				});
			}
		},
		...defaultContextItems
	];

	let mods: DeduplicatedMod<Mod>[] = $state([]);

	let maxCount: number = $state(20);
	let selectedMod: DeduplicatedMod<Mod> | null = $state(null);
	let foreignDownloadDialogOpen = $state(false);

	let installId: ModId;
	let unlistenFromQuery: UnlistenFn | undefined;

	const listedMods = $derived(
		mods.map((mod) => {
			if (mod.thunderstore) {
				return mod.thunderstore;
			} else if (mod.hexium) {
				return mod.hexium;
			} else {
				throw new Error('Mod is missing both thunderstore and hexium data');
			}
		})
	);

	onMount(() => {
		listen<DeduplicatedMod<Mod>[]>('mod_query_result', (evt) => {
			mods = evt.payload;
		}).then((unlisten) => {
			unlistenFromQuery = unlisten;
		});

		return () => {
			unlistenFromQuery?.();
			api.thunderstore.stopQuerying();
		};
	});

	let hasRefreshed = $state(false);
	let refreshing = false;

	function deduplicatedUuid(mod: DeduplicatedMod<Mod>) {
		return mod.thunderstore?.uuid ?? mod.hexium?.uuid;
	}

	function findModByUuid(uuid: string | undefined) {
		return mods.find((mod) => deduplicatedUuid(mod) === uuid) ?? null;
	}

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		mods = await api.thunderstore.query({ ...modQuery.current, maxCount });
		if (selectedMod) {
			// isInstalled might have changed
			selectedMod = findModByUuid(deduplicatedUuid(selectedMod));
		}

		refreshing = false;
		hasRefreshed = true;
	}

	async function installLatest(mod: Mod) {
		await install({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid,
			backend: mod.backend
		});
	}

	async function doInstall() {
		await api.profile.install.mod(installId);
		await refresh();
	}

	async function install(id: ModId) {
		installId = id;
		const prefs = await api.prefs.get();
		if (shouldWarnForeignDownload(id, prefs)) {
			foreignDownloadDialogOpen = true;
		} else {
			await doInstall();
		}
	}

	function onModClicked(evt: MouseEvent, mod: Mod) {
		if (evt.ctrlKey) {
			//installLatest(mod);
		} else if (selectedMod && deduplicatedUuid(selectedMod) === mod.uuid) {
			selectedMod = null;
		} else {
			selectedMod = findModByUuid(mod.uuid);
		}
	}

	$effect(() => {
		profiles.active;
		// read all fields of modQuery.current to trigger the effect when any of them change
		JSON.stringify(modQuery.current);
		if (maxCount > 0) {
			untrack(() => refresh());
		}
	});

	let locked = $derived(profiles.activeLocked);
</script>

<div class="flex grow overflow-hidden">
	<div class="flex w-[60%] grow flex-col overflow-hidden px-4 pt-4">
		<ModListFilters {sortOptions} bind:queryArgs={modQuery.current} />

		{#if locked}
			<ProfileLockedBanner class="mb-1" />
		{/if}

		<ModList mods={listedMods} queryArgs={modQuery.current} bind:maxCount>
			{#snippet placeholder()}
				{#if hasRefreshed}
					<HelpCard title={m.browse_modList_content_1()} icon="mdi:store-search" class="mt-4">
						{m.browse_modList_content_2()}
					</HelpCard>
				{/if}
			{/snippet}

			{#snippet item({ mod })}
				<ModListItem
					{mod}
					{contextItems}
					selected={selectedMod !== null && deduplicatedUuid(selectedMod) === mod.uuid}
					locked={profiles.activeLocked}
					oninstall={() => installLatest(mod)}
					onclick={(evt) => onModClicked(evt, mod)}
				/>
			{/snippet}
		</ModList>
	</div>

	{#if selectedMod}
		<DeduplicatedModDetails
			mod={selectedMod}
			{locked}
			{contextItems}
			onclose={() => (selectedMod = null)}
		>
			{#snippet children({ mod })}
				<InstallModButton {mod} {install} {locked} />
			{/snippet}
		</DeduplicatedModDetails>
	{/if}
</div>

<ForeignDownloadDialog bind:open={foreignDownloadDialogOpen} onConfirm={doInstall} />
