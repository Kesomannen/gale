<script lang="ts">
	import * as api from '$lib/api';
	import type { SortBy, Mod, ModId } from '$lib/types';

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
	import translation from '$lib/state/translation.svelte';

	const sortOptions: SortBy[] = ['lastUpdated', 'newest', 'rating', 'downloads'];
	const contextItems = [...defaultContextItems];

	let mods: Mod[] = $state([]);

	let modList: ModList;
	let maxCount: number = $state(20);
	let selectedMod: Mod | null = $state(null);

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

	function isNonAscii(str: string): boolean {
		return /[^\x00-\x7F]/.test(str);
	}

	async function refresh() {
		if (refreshing) return;
		refreshing = true;

		// If search contains non-ASCII (e.g. Chinese, Japanese, etc.), filter on frontend
		const queryForBackend = { ...modQuery.current };
		if (queryForBackend.searchTerm && isNonAscii(queryForBackend.searchTerm)) {
			queryForBackend.searchTerm = '';
		}

		mods = await api.thunderstore.query({ ...queryForBackend, maxCount });
		if (selectedMod) {
			selectedMod = mods.find((mod) => mod.uuid === selectedMod!.uuid) ?? null;
		}

		refreshing = false;
		hasRefreshed = true;
	}

	async function installLatest(mod: Mod) {
		await install({
			packageUuid: mod.uuid,
			versionUuid: mod.versions[0].uuid
		});
	}

	async function install(id: ModId) {
		await api.profile.install.mod(id);
		await refresh();
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

	let lastTranslateRequest = $state(0);

	$effect(() => {
		if (translation.translateRequest > lastTranslateRequest) {
			lastTranslateRequest = translation.translateRequest;
			if (mods.length > 0) {
				translation.translateMods(mods);
			}
		}
	});

	let lastAutoTranslatedCount = $state(0);

	$effect(() => {
		if (mods.length > 0 && translation.prefs?.enabled && translation.prefs?.apiKey && translation.prefs?.apiUrl && mods.length !== lastAutoTranslatedCount) {
			lastAutoTranslatedCount = mods.length;
			translation.translateMods(mods);
		}
	});

	let displayMods = $derived.by(() => {
		const searchTerm = modQuery.current.searchTerm?.toLowerCase().trim();
		if (!searchTerm) return mods;

		return mods.filter((mod) => {
			// Check original name/description
			const nameMatch = mod.name.toLowerCase().replace(/_/g, ' ').includes(searchTerm);
			const descMatch = mod.description?.toLowerCase().includes(searchTerm);
			if (nameMatch || descMatch) return true;

			// Check translation
			const translated = translation.getTranslation(mod.uuid);
			if (translated) {
				const transNameMatch = translated.name.toLowerCase().includes(searchTerm);
				const transDescMatch = translated.description?.toLowerCase().includes(searchTerm);
				if (transNameMatch || transDescMatch) return true;
			}

			return false;
		});
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
			mods={displayMods}
			queryArgs={modQuery.current}
			bind:this={modList}
			bind:maxCount
			bind:selected={selectedMod}
		>
			{#snippet placeholder()}
				{#if hasRefreshed}
					<div class="mt-4 text-lg">{m.browse_modList_content_1()}</div>
					<div class="text-primary-400">{m.browse_modList_content_2()}</div>
				{/if}
			{/snippet}

			{#snippet item({ mod, isSelected })}
				<ModListItem
					{mod}
					selected={isSelected}
					{contextItems}
					locked={profiles.activeLocked}
					oninstall={() => installLatest(mod)}
					onclick={(evt) => onModClicked(evt, mod)}
				/>
			{/snippet}
		</ModList>
	</div>

	{#if selectedMod}
		<ModDetails {locked} mod={selectedMod} {contextItems} onclose={() => (selectedMod = null)}>
			<InstallModButton mod={selectedMod} {install} {locked} />
		</ModDetails>
	{/if}
</div>
