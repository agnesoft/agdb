import styles from "./intro.module.scss";

export const Intro = () => {
    return (
        <div className={styles.intro}>
            <h1>agdb</h1>
            <p className={styles.subheading}>The graph database.</p>
            <p>
                <a href="/docs/guides/quickstart">Quickstart</a>
                <a href="/blog/blog">Why agdb?</a>
            </p>
        </div>
    );
};
