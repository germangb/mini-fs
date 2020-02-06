(function() {var implementors = {};
implementors["mini_fs"] = [{"text":"impl&lt;'_&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"mini_fs/struct.Entries.html\" title=\"struct mini_fs::Entries\">Entries</a>&lt;'_&gt;","synthetic":false,"types":["mini_fs::store::Entries"]}];
implementors["tar"] = [{"text":"impl&lt;'a, R:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"tar/struct.Entries.html\" title=\"struct tar::Entries\">Entries</a>&lt;'a, R&gt;","synthetic":false,"types":["tar::archive::Entries"]},{"text":"impl&lt;'entry&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"tar/struct.PaxExtensions.html\" title=\"struct tar::PaxExtensions\">PaxExtensions</a>&lt;'entry&gt;","synthetic":false,"types":["tar::pax::PaxExtensions"]}];
implementors["xattr"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"xattr/struct.XAttrs.html\" title=\"struct xattr::XAttrs\">XAttrs</a>","synthetic":false,"types":["xattr::sys::linux_macos::XAttrs"]}];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        })()