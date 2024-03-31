<script lang="ts">
	import type { ConfigFile, ConfigSection } from "$lib/models";
	import Icon from "@iconify/svelte";
	import { Button, Collapsible } from "bits-ui";
	import { quadOut } from "svelte/easing";
	import { slide } from "svelte/transition";

    export let file: ConfigFile;
    export let selectedSection: ConfigSection | undefined;
    export let onClick: (file: ConfigFile, section: ConfigSection) => void;

    let open = false;
</script>

<Collapsible.Root bind:open={open}>
    <Collapsible.Trigger class="pl-3 pr-6 text-slate-200 hover:text-slate-100 font-semibold flex items-center justify-between"> 
        <Icon
            icon="mdi:expand-more"
            class="text-xl {open ? 'rotate-180' : 'rotate-0'} transition-all mr-1"
        />
        {file.name}
    </Collapsible.Trigger>
    <Collapsible.Content
        class="flex flex-col mt-0.5" 
        transition={slide}
        transitionConfig={{ duration: 150, easing: quadOut }}
    >
        {#each file.sections as section}
            <Button.Root 
                class="text-sm px-8 py-0.5 text-left
                      {selectedSection === section ? 'bg-gray-600 text-slate-200 font-medium' : 'text-slate-300 hover:bg-gray-600 hover:text-slate-200 font-light'}"
                on:click={() => onClick(file, section)}
            >
                {section.name}
            </Button.Root>
        {/each}
    </Collapsible.Content>
</Collapsible.Root>