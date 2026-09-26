<script lang="ts">
	import * as api from '$lib/api';
	import { m } from '$lib/paraglide/messages';
	import type { AvailableUpdate, Mod, ModId, ModpackChange } from '$lib/types';
	import { formatModName } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import Button from '../ui/Button.svelte';
	import Checkbox from '../ui/Checkbox.svelte';
	import Checklist from '../ui/Checklist.svelte';
	import ConfirmDialog from '../ui/ConfirmDialog.svelte';
	import IconButton from '../ui/IconButton.svelte';
	import ModCard from '../ui/ModCard.svelte';
	import Tooltip from '../ui/Tooltip.svelte';
	import ModInfoDialog from './ModInfoDialog.svelte';

	type Props = {
		onUpdated: () => Promise<void>;
	};

	let { onUpdated }: Props = $props();

	let open = $state(false);
	let name = $state('');
	let from = $state('');
	let to = $state('');
	let target: ModId | null = null;
	let changes: ModpackChange[] = $state.raw([]);

	// map from package uuids to whether the change is selected
	let include = new SvelteMap<string, boolean>();
	let dontShowAgain = $state(false);

	let changelogOpen = $state(false);
	let changelog: string | null = $state(null);

	let selected = $derived(changes.filter(isSelected));

	/**
	 * Offers to change installed mods to the versions pinned by a modpack before updating it.
	 * Returns true if the dialog was opened, in which case it takes over the update.
	 */
	export async function openFor(
		mod: Mod,
		versionUuid: string | undefined,
		update: AvailableUpdate | undefined
	) {
		let newTarget: ModId | undefined;
		let newTo: string | undefined;

		if (versionUuid) {
			newTarget = { packageUuid: mod.uuid, versionUuid, backend: mod.backend };
			newTo = mod.versions.find((version) => version.uuid === versionUuid)?.name;
		} else {
			newTarget = update?.updatedId;
			newTo = update?.new;
		}

		if (!newTarget || !newTo) return false;

		let prefs = await api.prefs.get();
		if (!prefs.promptModpackVersions) return false;

		let newChanges: ModpackChange[];
		try {
			// this is empty for mods which aren't modpacks
			newChanges = await api.profile.update.modpackChanges(newTarget);
		} catch {
			return false; // fall back to updating only the modpack
		}

		if (!newChanges.some((change) => change.recommended)) return false;

		target = newTarget;
		changes = newChanges;
		name = formatModName(mod.name);
		from = mod.version ?? m.unknown();
		to = newTo;
		include.clear();
		dontShowAgain = false;
		open = true;

		return true;
	}

	function isSelected(change: ModpackChange) {
		return include.get(change.id.packageUuid) ?? change.recommended;
	}

	function hint(change: ModpackChange) {
		if (change.conflict !== null) {
			let modpack = `${formatModName(change.conflict.name)} ${change.conflict.version}`;
			return m.modpackUpdateDialog_conflict({ modpack });
		}

		if (change.kind === 'upgrade' && change.ignored) return m.modpackUpdateDialog_ignored();
		if (change.kind === 'rollback') return m.modpackUpdateDialog_rollback();
		if (change.kind === 'ahead') return m.modpackUpdateDialog_ahead();
		return null;
	}

	async function confirm(ids: ModId[]) {
		if (target === null) return;

		if (dontShowAgain) {
			let prefs = await api.prefs.get();
			prefs.promptModpackVersions = false;
			await api.prefs.set(prefs);
		}

		open = false;

		try {
			await api.profile.update.changeModVersions([target, ...ids]);
		} finally {
			await onUpdated();
		}
	}

	async function openChangelog(change: ModpackChange) {
		changelog = await api.thunderstore.getMarkdown(change.id, 'changelog');
		changelogOpen = true;
	}
</script>

<ConfirmDialog title={m.modpackUpdateDialog_title()} bind:open>
	{m.modpackUpdateDialog_content({ name, from, to })}

	<!-- leave room for the rest of the dialog, even in a small window -->
	<Checklist
		title={m.modpackUpdateDialog_changes_title()}
		items={changes}
		class="mt-3 flex max-h-[max(8rem,min(24rem,calc(85vh_-_20rem)))] flex-col"
		maxHeight="sm"
		get={(change, _) => isSelected(change)}
		set={(change, _, value) => include.set(change.id.packageUuid, value)}
	>
		{#snippet item({ item: change })}
			{@const text = hint(change)}

			<ModCard fullName={change.fullName} showVersion={false} backend={change.id.backend} />

			<div class="grow"></div>

			{#if text !== null}
				<Tooltip {text}>
					<Icon
						icon={change.conflict !== null ? 'mdi:alert-circle' : 'mdi:information'}
						class="text-accent-600 dark:text-accent-400 mr-2 text-lg"
					/>
				</Tooltip>
			{/if}

			<span class="text-primary-500 dark:text-primary-400 shrink-0 pl-1">{change.old}</span>
			<Icon
				icon={change.kind === 'upgrade' ? 'mdi:arrow-top-right' : 'mdi:arrow-bottom-right'}
				class="text-primary-500 dark:text-primary-400 mx-1.5 shrink-0 text-lg"
			/>
			<span class="text-accent-600 dark:text-accent-400 shrink-0 text-lg font-semibold"
				>{change.new}</span
			>

			<IconButton
				class="ml-3"
				icon="mdi:file-document"
				label={m.modpackUpdateDialog_changelog()}
				onclick={() => openChangelog(change)}
			/>
		{/snippet}
	</Checklist>

	<div class="mt-3 flex items-center">
		<Checkbox id="modpackupdatedontshowagain" bind:checked={dontShowAgain} />

		<label class="ml-3" for="modpackupdatedontshowagain">
			{m.modpackUpdateDialog_dontShowAgain()}
		</label>
	</div>

	{#snippet buttons()}
		<Button color="primary" onclick={() => confirm([])}>
			{m.modpackUpdateDialog_button_modpackOnly()}
		</Button>
		<Button
			color="accent"
			icon="mdi:download"
			disabled={selected.length === 0}
			onclick={() => confirm(selected.map((change) => change.id))}
		>
			{m.modpackUpdateDialog_button_confirm({ count: selected.length })}
		</Button>
	{/snippet}

	<ModInfoDialog bind:open={changelogOpen} content={changelog} type="changelog" />
</ConfirmDialog>
