import styles from "./footer.module.scss";
import Link from "next/link";

export default function Footer() {
    return (
        <footer
            className={
                styles.footer +
                " nx-mx-auto nx-max-w-[90rem] nx-text-gray-600 dark:nx-text-gray-400"
            }
        >
            <div className={styles.footerLinks}>
                <div className={styles.footerColumn}>
                    <Link href="/docs">Documentation</Link>
                    <Link href="/contact">Contact</Link>
                    <Link href="/contact">Contact</Link>
                </div>
                <div className={styles.footerColumn}>
                    <Link href="/about">About</Link>
                    <Link href="/contact">Contact</Link>
                    <Link href="/contact">Contact</Link>
                </div>
                <div className={styles.footerColumn}>
                    <Link href="/about">About</Link>
                    <Link href="/contact">Contact</Link>
                    <Link href="/contact">Contact</Link>
                </div>
            </div>
            <div className={styles.copyright}>
                Copyright @ {new Date().getFullYear()} agdb
            </div>
        </footer>
    );
}
