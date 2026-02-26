<script lang="ts">
	import type { MissingProfileAction, ProfileInfo } from '$lib/types';
	import PathField from '../ui/PathField.svelte';
	import Select from '../ui/Select.svelte';
	import { open } from '@tauri-apps/plugin-dialog';

	type Props = {
		profile: ProfileInfo;
		value: MissingProfileAction | null;
		onChanged: (action: MissingProfileAction) => void;
	};

	let { profile, value, onChanged }: Props = $props();

	let showPathField = $derived(value?.type === 'locate');
	let pathValue = $derived(value?.type === 'locate' ? value.newPath : null);

	function browseNewPath() {
		open({
			directory: true,
			defaultPath: pathValue ?? undefined,
			title: `Locate profile folder for ${profile.name}`
		}).then((result) => {
			if (result === null) return;
			onChanged({ type: 'locate', newPath: result });
		});
	}
</script>

<div class="text-primary-300 even:bg-primary-900 space-y-2 px-4 py-2">
	<div class="flex items-center">
		<div class="w-60 truncate">{profile.name}</div>
		<Select
			type="single"
			triggerClass="grow"
			placeholder="Select an action"
			value={value?.type}
			onValueChange={(value) => {
				if (value === 'locate') {
					showPathField = true;
				} else {
					onChanged({ type: 'delete' });
				}
			}}
			items={[
				{ label: 'Locate', value: 'locate' },
				{ label: 'Delete', value: 'delete' }
			]}
		/>
	</div>

	{#if showPathField}
		<PathField value={pathValue} onclick={browseNewPath}
			>Locate the new folder of this profile.</PathField
		>
	{/if}
</div>
