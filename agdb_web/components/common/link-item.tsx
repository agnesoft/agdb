import Link from "next/link";
import React from "react";
import { useI18n } from "@/hooks/i18n";

type LinkItemProps = {
    i18nKey: string;
    children?: React.ReactNode;
};

export const LinkItem = ({ i18nKey, children }: LinkItemProps) => {
    const { t } = useI18n();
    return (
        <Link href={t(`url.${i18nKey}`)}>
            {t(`link.${i18nKey}`)}
            {children}
        </Link>
    );
};
