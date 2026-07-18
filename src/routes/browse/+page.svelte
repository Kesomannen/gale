<script lang="ts">
	import * as api from '$lib/api';
	import { type SortBy, type Mod, type ModId, Backend, type ModContextItem } from '$lib/types';

	import ModList from '$lib/components/mod-list/ModList.svelte';

	import { onMount } from 'svelte';
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
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { pushInfoToast } from '$lib/toast';
	import HelpCard from '$lib/components/ui/HelpCard.svelte';
	import games from '$lib/state/game.svelte';

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

	let mods: Mod[] = $state([]);

	let modList: ModList;
	let maxCount: number = $state(20);
	let selectedMod: Mod | null = $state(null);
	let installDialogOpen = $state(false);
	let loading = $state(false);
	let warnNoRemind = $state(false);

	let installId: ModId;
	let unlistenFromQuery: UnlistenFn | undefined;

	onMount(() => {
		listen<Mod[]>('mod_query_result', (evt) => {
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

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		mods = await api.thunderstore.query({ ...modQuery.current, maxCount });
		if (selectedMod) {
			// isInstalled might have changed
			selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
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
		if (warnNoRemind) {
			let prefs = await api.prefs.get();
			prefs.backendSkipConfirm = true;
			await api.prefs.set(prefs);
		}
		installDialogOpen = false;
		loading = true;
		await api.profile.install.mod(installId);
		await refresh();
	}

	async function install(id: ModId) {
		installId = id;
		if (
			id.backend !== Backend.Thunderstore &&
			!(await api.prefs.get()).backendSkipConfirm &&
			games.activeBackends.length > 1
		) {
			installDialogOpen = true;
		} else {
			await doInstall();
		}
	}

	function onModClicked(evt: MouseEvent, mod: Mod) {
		if (evt.ctrlKey) {
			installLatest(mod);
		} else {
			modList.selectMod(mod);
		}
	}

	$effect(() => {
		if (maxCount > 0) {
			modQuery.current;
			profiles.active;
			refresh();
		}
	});

	$effect(() => {
		loading = false;
	});

	let locked = $derived(profiles.activeLocked);
</script>

<div class="flex grow overflow-hidden">
	<div class="flex w-[60%] grow flex-col overflow-hidden px-4 pt-4">
		<ModListFilters {sortOptions} queryArgs={modQuery.current} />

		{#if locked}
			<ProfileLockedBanner class="mb-1" />
		{/if}

		<ModList
			{mods}
			queryArgs={modQuery.current}
			bind:this={modList}
			bind:maxCount
			bind:selected={selectedMod}
		>
			{#snippet placeholder()}
				{#if hasRefreshed}
					<HelpCard title={m.browse_modList_content_1()} icon="mdi:store-search" class="mt-4">
						{m.browse_modList_content_2()}
					</HelpCard>
				{/if}
			{/snippet}

			{#snippet item({ mod, isSelected })}
				<ModListItem
					{mod}
					{contextItems}
					selected={isSelected}
					locked={profiles.activeLocked}
					oninstall={() => installLatest(mod)}
					onclick={(evt) => onModClicked(evt, mod)}
				/>
			{/snippet}
		</ModList>
	</div>

	{#if selectedMod}
		<ModDetails {locked} mod={selectedMod} {contextItems} onclose={() => (selectedMod = null)}>
			<InstallModButton mod={selectedMod} {install} {locked} {loading} />
		</ModDetails>
	{/if}

	<ConfirmDialog title={m.otherServer_warn_title()} bind:open={installDialogOpen}>
		{m.otherServer_warn_content()}
		<div class="my-5 flex items-center">
			<Checkbox id="neverwarninstall" bind:checked={warnNoRemind} />
			<label class="ml-3" for="neverwarninstall">
				{m.otherServer_warn_noremind()}
			</label>
		</div>

		{#snippet buttons()}
			<Button color="accent" icon="mdi:download" onclick={doInstall}>
				{m.installModButton_button_install()}
			</Button>
		{/snippet}
	</ConfirmDialog>
</div>
