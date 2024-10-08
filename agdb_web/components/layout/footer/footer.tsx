import styles from "./footer.module.scss";
import LinkItem from "@/components/common/link-item";

export const Footer = () => {
    return (
        <footer
            className={
                styles.footer +
                " nx-mx-auto nx-max-w-[90rem] nx-text-gray-600 dark:nx-text-gray-400"
            }
        >
            <div className={styles.footerLinks}>
                <div className={styles.footerColumn}>
                    <LinkItem i18nKey="docs" />
                    <LinkItem i18nKey="api" />
                    <LinkItem i18nKey="blog" />
                </div>
                <div className={styles.footerColumn}>
                    <LinkItem i18nKey="license" />
                    <LinkItem i18nKey="enterprise" />
                </div>
                <div className={styles.footerColumn}>
                    <LinkItem i18nKey="about" />
                    <LinkItem i18nKey="contact" />
                    <LinkItem i18nKey="license" />
                </div>
            </div>
            <div className={styles.copyright}>
                Copyright @ {new Date().getFullYear()} agdb
            </div>
        </footer>
    );
};
