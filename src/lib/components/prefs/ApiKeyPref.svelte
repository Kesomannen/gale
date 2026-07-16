<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import * as api from '$lib/api';
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import { apiKeyDialog } from '$lib/state/misc.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { Backend } from '$lib/types';

	type Props = {
		backend: Backend;
	};

	let { backend }: Props = $props();

	let hasToken = $state(false);

	async function refresh() {
		hasToken = await api.thunderstore.hasToken(backend);
	}

	function openDialog() {
		apiKeyDialog.open = true;
		apiKeyDialog.backend = backend;
	}

	$effect(() => {
		apiKeyDialog.open;
		apiKeyDialog.backend;
		refresh();
	});
</script>

<div class="flex items-center">
	<Label>{m.apiKeyPref_title({ backend })}</Label>

	<Info>
		{m.apiKeyPref_content({ backend })}
	</Info>

	<button
		class="group bg-primary-900 hover:border-primary-500 relative flex grow items-center truncate rounded-lg border border-transparent px-3 py-1 text-right"
		onclick={openDialog}
	>
		<div class="mr-2 rounded-sm">
			<Icon
				icon={hasToken ? 'mdi:key' : 'mdi:key-remove'}
				class="text-primary-300 group-hover:text-primary-200 align-middle"
			/>
		</div>

		<div class="text-primary-300 group-hover:text-primary-200 truncate">
			{m[`apiKeyPref_hasToken_${hasToken ? 'has' : 'no'}`]()}
		</div>
	</button>
</div>
