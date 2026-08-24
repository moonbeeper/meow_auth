<script lang="ts">
    import Button from "$comps/button.svelte";
    import * as Dialog from "$comps/dialog/";
    import AuditLogItem2 from "$comps/items/auditLogItem2.svelte";
    import LoadingBox from "$comps/loadingBox.svelte";
    import * as LogItem from "$comps/logItem";
    import SettingsPanel from "$comps/settingsPanel.svelte";
    import { createCurrentUserAuditLog } from "$lib/api/user/user";
    import { actionCategory, actionLabel, intoAuditAction } from "$lib/auditLog";
    import { actionDate } from "$lib/common";

    import ProfilePanel from "./profilePanel.svelte";

    const auditLogQuery = createCurrentUserAuditLog();

    const recentAuditLog = $derived.by(() => {
        if (auditLogQuery.data?.status != 200) return [];
        const data = auditLogQuery.data.data.data ?? [];

        return data.slice(0, 8);
    });

    let isAuditLogLong = $derived.by(() => {
        return recentAuditLog.length >= 8;
    });
</script>

<ProfilePanel />

<SettingsPanel
    title="Audit log"
    description="A little trail of your recent activity on your account"
>
    {#if auditLogQuery.isLoading || auditLogQuery.isError}
        <LoadingBox loaded={!auditLogQuery.isLoading} hasError={auditLogQuery.isError} />
    {:else if auditLogQuery.isSuccess}
        <LogItem.Container>
            {#each recentAuditLog as logItem (logItem.id)}
                <AuditLogItem2
                    title={actionLabel(intoAuditAction(logItem.action))}
                    tag={actionCategory(intoAuditAction(logItem.action))}
                    who={logItem.was_self ? "you" : "admin"}
                    when={actionDate(logItem.created_at)}
                />
            {/each}
        </LogItem.Container>
    {/if}
    {#if isAuditLogLong}
        <Button href="/me/audit-log">View all</Button>
    {/if}
</SettingsPanel>

<SettingsPanel
    title="Delete account"
    description="Delete your account and everything associated with it, including authorized apps and created apps."
    negative
>
    <Button negative>Delete account</Button>
    <!-- <Dialog.Root width="large">
        {#snippet trigger({ props })}
            <Button negative {...props}>Delete account</Button>
        {/snippet}

        {#snippet actions()}
            <Button>AntiPoop</Button>
            <Button primary>Poop</Button>
        {/snippet} -->

    <!-- hi -->
    <!-- Lorem ipsum dolor sit amet, consectetur adipiscing elit. Quisque sollicitudin mauris maximus elit
        sagittis, nec lobortis ligula elementum. Nam iaculis, urna nec lobortis posuere, eros urna venenatis
        eros, vel accumsan turpis nunc vitae enim. Maecenas et lorem lectus. Vivamus iaculis tortor eget
        ante placerat, nec posuere nisl tincidunt. Cras condimentum ante in accumsan ultricies. Morbi
        quis porta est, sit amet congue augue. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        Ut consequat nunc id quam tempus, id tincidunt neque venenatis. Mauris fringilla tempor est, vitae
        fermentum enim elementum vitae. Nullam eleifend odio ut porta efficitur. Phasellus luctus tempus
        posuere. Curabitur scelerisque bibendum faucibus. Duis rhoncus nunc est, at pharetra eros tristique
        a. Nam sodales turpis lectus, quis faucibus felis fermentum in. Curabitur vel velit vel eros laoreet
        pharetra. Aenean in facilisis sapien, eu porttitor ex. Donec ultrices ac arcu ut lobortis. Pellentesque
        vitae rutrum orci. Etiam pretium et enim sit amet scelerisque. Nulla sed odio nec lorem dapibus
        condimentum at sagittis quam. Sed in ornare ex, sed luctus sem. Mauris a est tellus. Sed fringilla
        est ac urna aliquet, eget condimentum felis vulputate. Sed sagittis eros non mauris sodales molestie.
        Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Nullam
        ante leo, condimentum sed lectus non, rutrum octopodes urna. Mauris neque ante, interdum molestie
        tellus pharetra, eleifend dapibus justo. Sed at diam ligula. Donec dapibus ipsum quis elit euismod,
        sed suscipit eros euismod. Aliquam pretium felis quis risus luctus fringilla. Ut purus lacus,
        mattis a turpis eget, sollicitudin pellentesque neque. Nunc sodales quis ante quis porttitor.
        Vestibulum ornare lacinia ante. Donec a nisi nec arcu aliquam pretium in nec nunc. Donec fringilla
        erat vitae viverra feugiat. Sed non odio vel ipsum porttitor maximus. Donec id eleifend lectus.
        Proin varius felis sit amet neque eleifend, vitae porttitor ligula commodo. Vivamus felis quam,
        porttitor a justo sit amet, placerat ultricies nisl. Suspendisse potenti. Maecenas non consequat
        lorem, eu porta ante. Pellentesque elementum diam sapien, nec ultrices risus convallis eget. Nam
        pharetra dolor at dictum tempor. Quisque ut est a ligula hendrerit sodales. Curabitur ornare a
        nulla in laoreet. Maecenas semper mi egestas, dignissim nisi et, elementum neque.
    </Dialog.Root> -->
</SettingsPanel>
