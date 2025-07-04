<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import * as api from '$lib/api';
	import { apiKeyPopupOpen } from './ApiKeyPopup.svelte';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		field?: Snippet;
	};

	let { field }: Props = $props();

	let hasToken = $state(false);

	async function refresh() {
		hasToken = await api.thunderstore.hasToken();
	}

	$effect(() => {
		$apiKeyPopupOpen;
		refresh();
	});
</script>

<div class="flex items-center">
	<Label>Thunderstore API token</Label>

	<Info>
		Thunderstore API token to use for modpack publishing. Once this is set, you will <b>not</b> be able
		to view the token again.
	</Info>

	<button
		class="group bg-primary-900 hover:border-primary-500 relative flex grow items-center truncate rounded-lg border border-transparent px-3 py-1 text-right"
		onclick={() => ($apiKeyPopupOpen = true)}
	>
		<div class="mr-2 rounded-sm">
			<Icon
				icon={hasToken ? 'mdi:key' : 'mdi:key-remove'}
				class="text-primary-300 group-hover:text-primary-200 align-middle"
			/>
		</div>

		<div class="text-primary-300 group-hover:text-primary-200 truncate">
			{hasToken ? 'Click to override token' : 'Not set'}
		</div>

		{@render field?.()}
	</button>
</div>
