import Image from "next/image";
import styles from "./logo.module.scss";

export const Logo = () => {
    return (
        <>
            <Image
                src="/images/logo.svg"
                alt="logo"
                width="70"
                height="70"
                className={styles.logo}
            />
            <span style={{ marginLeft: ".8rem" }}>agdb</span>
        </>
    );
};
