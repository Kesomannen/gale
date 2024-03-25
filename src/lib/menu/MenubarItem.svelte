<script lang="ts">
	import { Menubar } from 'bits-ui';

	export let onClick: () => void;
</script>

<Menubar.Item
	class="pl-4 pr-10 py-1 text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
	on:click={onClick}
>
	<slot />
</Menubar.Item>
