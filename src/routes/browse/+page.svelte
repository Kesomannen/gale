<script lang="ts">
	import * as api from '$lib/api';
	import {
		type SortBy,
		type Mod,
		type ModId,
		type ModContextItem,
		type BrowsedMod
	} from '$lib/types';

	import ModList from '$lib/components/mod-list/ModList.svelte';

	import { onMount, untrack } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import ModListItem from '$lib/components/mod-list/ModListItem.svelte';
	import ProfileLockedBanner from '$lib/components/mod-list/ProfileLockedBanner.svelte';
	import ModListFilters from '$lib/components/mod-list/ModListFilters.svelte';
	import { defaultContextItems } from '$lib/context';
	import InstallModButton from '$lib/components/mod-list/InstallModButton.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { modQuery } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import { pushInfoToast } from '$lib/toast';
	import HelpCard from '$lib/components/ui/HelpCard.svelte';
	import ForeignDownloadDialog from '$lib/components/dialogs/ForeignDownloadDialog.svelte';
	import {
		extractDeduplicatedMod,
		getPreferredBackend,
		shouldWarnForeignDownload
	} from '$lib/util';
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

	let mods: BrowsedMod[] = $state([]);

	let maxCount: number = $state(20);
	let selectedMod: BrowsedMod | null = $state(null);
	let foreignDownloadDialogOpen = $state(false);

	let installId: ModId;
	let unlistenFromQuery: UnlistenFn | undefined;

	onMount(() => {
		listen<BrowsedMod[]>('mod_query_result', (evt) => {
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

	function itemUuid(item: BrowsedMod) {
		return item.data.thunderstore?.uuid ?? item.data.hexium?.uuid;
	}

	function findItemByUuid(uuid: string | undefined) {
		return mods.find((mod) => itemUuid(mod) === uuid) ?? null;
	}

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		mods = await api.thunderstore.query({ ...modQuery.current, maxCount });
		if (selectedMod) {
			// isInstalled might have changed
			selectedMod = findItemByUuid(itemUuid(selectedMod));
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
			installLatest(mod);
		} else if (selectedMod && itemUuid(selectedMod) === mod.uuid) {
			selectedMod = null;
		} else {
			selectedMod = findItemByUuid(mod.uuid);
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

		<ModList {mods} queryArgs={modQuery.current} bind:maxCount>
			{#snippet placeholder()}
				{#if hasRefreshed}
					<HelpCard title={m.browse_modList_content_1()} icon="mdi:store-search" class="mt-4">
						{m.browse_modList_content_2()}
					</HelpCard>
				{/if}
			{/snippet}

			{#snippet item({ mod })}
				{@const shownMod = extractDeduplicatedMod(mod.data, getPreferredBackend(mod.data))!}
				{@const combinedDownloads =
					(mod.data.hexium?.downloads ?? 0) + (mod.data.thunderstore?.downloads ?? 0)}

				<ModListItem
					{contextItems}
					{combinedDownloads}
					mod={shownMod}
					isInstalled={mod.isInstalled}
					selected={selectedMod !== null && itemUuid(selectedMod) === shownMod.uuid}
					locked={profiles.activeLocked}
					oninstall={() => installLatest(shownMod)}
					onclick={(evt) => onModClicked(evt, shownMod)}
				/>
			{/snippet}
		</ModList>
	</div>

	{#if selectedMod}
		<DeduplicatedModDetails
			{locked}
			mod={selectedMod.data}
			{contextItems}
			onclose={() => (selectedMod = null)}
		>
			{#snippet children({ mod })}
				<InstallModButton {mod} isInstalled={selectedMod?.isInstalled} {install} {locked} />
			{/snippet}
		</DeduplicatedModDetails>
	{/if}
</div>

<ForeignDownloadDialog bind:open={foreignDownloadDialogOpen} onConfirm={doInstall} />
