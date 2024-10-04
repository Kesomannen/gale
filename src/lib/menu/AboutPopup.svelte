<script lang="ts">
	import Popup from '$lib/components/Popup.svelte';
	import Link from '$lib/components/Link.svelte';

	import Icon from '@iconify/svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { onMount } from 'svelte';

	export let open = false;

	let version = '';

	onMount(async () => {
		version = await getVersion();
	});
</script>

<Popup bind:open title="About">
	<div class="h-3" />
	<img src="logo.png" alt="Logo" class="float-right size-20" />
	<div>
		<h3 class="text-xl font-semibold text-white">Gale</h3>
		<p class="text-slate-300">
			Version {version}
			<br />
			GNU General Public License v3.0
		</p>
		<div class="mt-2 flex items-center gap-2">
			<Icon icon="mdi:github" class="inline text-xl text-white" />
			<Link href="https://github.com/Kesomannen/gale">GitHub</Link>
		</div>
		<div class="mt-1 flex items-center gap-2">
			<Icon icon="mdi:link" class="inline text-xl text-white" />
			<Link href="https://kesomannen.com/gale">Website</Link>
		</div>
	</div>
</Popup>
