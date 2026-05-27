pub mod open {
    //! Open file APIs.
    use crate::js::JsFuture;
    use crate::js::binding;
    use crate::js::converter::*;
    use crate::js::resource::ResourceId;
    use crate::js::resource::ResourceTableArc;
    use crate::prelude::*;
    use compact_str::ToCompactString;
    pub const APPEND: &str = "append";
    pub const CREATE: &str = "create";
    pub const CREATE_NEW: &str = "createNew";
    pub const READ: &str = "read";
    pub const TRUNCATE: &str = "truncate";
    pub const WRITE: &str = "write";
    pub const APPEND_DEFAULT: bool = false;
    pub const CREATE_DEFAULT: bool = false;
    pub const CREATE_NEW_DEFAULT: bool = false;
    pub const READ_DEFAULT: bool = false;
    pub const TRUNCATE_DEFAULT: bool = false;
    pub const WRITE_DEFAULT: bool = false;
    pub struct FsOpenOptions {
        #[builder(default = APPEND_DEFAULT)]
        pub append: bool,
        #[builder(default = CREATE_DEFAULT)]
        pub create: bool,
        #[builder(default = CREATE_NEW_DEFAULT)]
        pub create_new: bool,
        #[builder(default = READ_DEFAULT)]
        pub read: bool,
        #[builder(default = TRUNCATE_DEFAULT)]
        pub truncate: bool,
        #[builder(default = WRITE_DEFAULT)]
        pub write: bool,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for FsOpenOptions {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "append",
                "create",
                "create_new",
                "read",
                "truncate",
                "write",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.append,
                &self.create,
                &self.create_new,
                &self.read,
                &self.truncate,
                &&self.write,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "FsOpenOptions",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for FsOpenOptions {}
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for FsOpenOptions {}
    #[automatically_derived]
    impl ::core::clone::Clone for FsOpenOptions {
        #[inline]
        fn clone(&self) -> FsOpenOptions {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for FsOpenOptions {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for FsOpenOptions {
        #[inline]
        fn eq(&self, other: &FsOpenOptions) -> bool {
            self.append == other.append && self.create == other.create
                && self.create_new == other.create_new && self.read == other.read
                && self.truncate == other.truncate && self.write == other.write
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for FsOpenOptions {
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<bool>;
        }
    }
    #[allow(clippy::all)]
    /**Builder for [`FsOpenOptions`](struct.FsOpenOptions.html).
*/
    pub struct FsOpenOptionsBuilder {
        append: ::derive_builder::export::core::option::Option<bool>,
        create: ::derive_builder::export::core::option::Option<bool>,
        create_new: ::derive_builder::export::core::option::Option<bool>,
        read: ::derive_builder::export::core::option::Option<bool>,
        truncate: ::derive_builder::export::core::option::Option<bool>,
        write: ::derive_builder::export::core::option::Option<bool>,
    }
    #[automatically_derived]
    #[allow(clippy::all)]
    impl ::core::clone::Clone for FsOpenOptionsBuilder {
        #[inline]
        fn clone(&self) -> FsOpenOptionsBuilder {
            FsOpenOptionsBuilder {
                append: ::core::clone::Clone::clone(&self.append),
                create: ::core::clone::Clone::clone(&self.create),
                create_new: ::core::clone::Clone::clone(&self.create_new),
                read: ::core::clone::Clone::clone(&self.read),
                truncate: ::core::clone::Clone::clone(&self.truncate),
                write: ::core::clone::Clone::clone(&self.write),
            }
        }
    }
    #[allow(clippy::all)]
    #[allow(dead_code)]
    impl FsOpenOptionsBuilder {
        #[allow(unused_mut)]
        pub fn append(&mut self, value: bool) -> &mut Self {
            let mut new = self;
            new.append = ::derive_builder::export::core::option::Option::Some(value);
            new
        }
        #[allow(unused_mut)]
        pub fn create(&mut self, value: bool) -> &mut Self {
            let mut new = self;
            new.create = ::derive_builder::export::core::option::Option::Some(value);
            new
        }
        #[allow(unused_mut)]
        pub fn create_new(&mut self, value: bool) -> &mut Self {
            let mut new = self;
            new.create_new = ::derive_builder::export::core::option::Option::Some(value);
            new
        }
        #[allow(unused_mut)]
        pub fn read(&mut self, value: bool) -> &mut Self {
            let mut new = self;
            new.read = ::derive_builder::export::core::option::Option::Some(value);
            new
        }
        #[allow(unused_mut)]
        pub fn truncate(&mut self, value: bool) -> &mut Self {
            let mut new = self;
            new.truncate = ::derive_builder::export::core::option::Option::Some(value);
            new
        }
        #[allow(unused_mut)]
        pub fn write(&mut self, value: bool) -> &mut Self {
            let mut new = self;
            new.write = ::derive_builder::export::core::option::Option::Some(value);
            new
        }
        /**Builds a new `FsOpenOptions`.

# Errors

If a required field has not been initialized.
*/
        pub fn build(
            &self,
        ) -> ::derive_builder::export::core::result::Result<
            FsOpenOptions,
            FsOpenOptionsBuilderError,
        > {
            Ok(FsOpenOptions {
                append: match self.append {
                    Some(ref value) => {
                        ::derive_builder::export::core::clone::Clone::clone(value)
                    }
                    None => APPEND_DEFAULT,
                },
                create: match self.create {
                    Some(ref value) => {
                        ::derive_builder::export::core::clone::Clone::clone(value)
                    }
                    None => CREATE_DEFAULT,
                },
                create_new: match self.create_new {
                    Some(ref value) => {
                        ::derive_builder::export::core::clone::Clone::clone(value)
                    }
                    None => CREATE_NEW_DEFAULT,
                },
                read: match self.read {
                    Some(ref value) => {
                        ::derive_builder::export::core::clone::Clone::clone(value)
                    }
                    None => READ_DEFAULT,
                },
                truncate: match self.truncate {
                    Some(ref value) => {
                        ::derive_builder::export::core::clone::Clone::clone(value)
                    }
                    None => TRUNCATE_DEFAULT,
                },
                write: match self.write {
                    Some(ref value) => {
                        ::derive_builder::export::core::clone::Clone::clone(value)
                    }
                    None => WRITE_DEFAULT,
                },
            })
        }
        /// Create an empty builder, with all fields set to `None` or `PhantomData`.
        fn create_empty() -> Self {
            Self {
                append: ::derive_builder::export::core::default::Default::default(),
                create: ::derive_builder::export::core::default::Default::default(),
                create_new: ::derive_builder::export::core::default::Default::default(),
                read: ::derive_builder::export::core::default::Default::default(),
                truncate: ::derive_builder::export::core::default::Default::default(),
                write: ::derive_builder::export::core::default::Default::default(),
            }
        }
    }
    impl ::derive_builder::export::core::default::Default for FsOpenOptionsBuilder {
        fn default() -> Self {
            Self::create_empty()
        }
    }
    ///Error type for FsOpenOptionsBuilder
    #[non_exhaustive]
    pub enum FsOpenOptionsBuilderError {
        /// Uninitialized field
        UninitializedField(&'static str),
        /// Custom validation error
        ValidationError(::derive_builder::export::core::string::String),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for FsOpenOptionsBuilderError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                FsOpenOptionsBuilderError::UninitializedField(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UninitializedField",
                        &__self_0,
                    )
                }
                FsOpenOptionsBuilderError::ValidationError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ValidationError",
                        &__self_0,
                    )
                }
            }
        }
    }
    impl ::derive_builder::export::core::convert::From<
        ::derive_builder::UninitializedFieldError,
    > for FsOpenOptionsBuilderError {
        fn from(s: ::derive_builder::UninitializedFieldError) -> Self {
            Self::UninitializedField(s.field_name())
        }
    }
    impl ::derive_builder::export::core::convert::From<
        ::derive_builder::export::core::string::String,
    > for FsOpenOptionsBuilderError {
        fn from(s: ::derive_builder::export::core::string::String) -> Self {
            Self::ValidationError(s)
        }
    }
    impl ::derive_builder::export::core::fmt::Display for FsOpenOptionsBuilderError {
        fn fmt(
            &self,
            f: &mut ::derive_builder::export::core::fmt::Formatter,
        ) -> ::derive_builder::export::core::fmt::Result {
            match self {
                Self::UninitializedField(ref field) => {
                    f.write_fmt(format_args!("`{0}` must be initialized", field))
                }
                Self::ValidationError(ref error) => {
                    f.write_fmt(format_args!("{0}", error))
                }
            }
        }
    }
    impl std::error::Error for FsOpenOptionsBuilderError {}
    impl crate::js::converter::ToV8 for FsOpenOptions {
        fn to_v8<'s>(
            &self,
            scope: &mut v8::PinScope<'s, '_>,
        ) -> v8::Local<'s, v8::Value> {
            let obj = v8::Object::new(scope);
            {
                let append_value = self.append.to_v8(scope);
                crate::js::binding::set_property_to(scope, obj, &"append", append_value);
            }
            {
                let create_value = self.create.to_v8(scope);
                crate::js::binding::set_property_to(scope, obj, &"create", create_value);
            }
            {
                let create_new_value = self.create_new.to_v8(scope);
                crate::js::binding::set_property_to(
                    scope,
                    obj,
                    &"createNew",
                    create_new_value,
                );
            }
            {
                let read_value = self.read.to_v8(scope);
                crate::js::binding::set_property_to(scope, obj, &"read", read_value);
            }
            {
                let truncate_value = self.truncate.to_v8(scope);
                crate::js::binding::set_property_to(
                    scope,
                    obj,
                    &"truncate",
                    truncate_value,
                );
            }
            {
                let write_value = self.write.to_v8(scope);
                crate::js::binding::set_property_to(scope, obj, &"write", write_value);
            }
            obj.into()
        }
    }
    impl crate::js::converter::FromV8 for FsOpenOptions {
        fn from_v8<'s>(
            scope: &mut v8::PinScope<'s, '_>,
            obj: v8::Local<'s, v8::Value>,
        ) -> Self {
            if true {
                if !(obj.is_object() || obj.is_object_template()) {
                    ::core::panicking::panic(
                        "assertion failed: obj.is_object() || obj.is_object_template()",
                    )
                }
            }
            let obj = obj.to_object(scope).unwrap();
            let mut builder = FsOpenOptionsBuilder::default();
            {
                let append_name = v8::String::new(scope, APPEND).unwrap();
                if true {
                    if !obj.has_own_property(scope, append_name.into()).unwrap_or(false)
                    {
                        ::core::panicking::panic(
                            "assertion failed: obj.has_own_property(scope, append_name.into()).unwrap_or(false)",
                        )
                    }
                }
                let append_value = obj.get(scope, append_name.into()).unwrap();
                builder
                    .append(
                        <bool as crate::js::converter::FromV8>::from_v8(
                            scope,
                            append_value,
                        ),
                    );
            }
            {
                let create_name = v8::String::new(scope, CREATE).unwrap();
                if true {
                    if !obj.has_own_property(scope, create_name.into()).unwrap_or(false)
                    {
                        ::core::panicking::panic(
                            "assertion failed: obj.has_own_property(scope, create_name.into()).unwrap_or(false)",
                        )
                    }
                }
                let create_value = obj.get(scope, create_name.into()).unwrap();
                builder
                    .create(
                        <bool as crate::js::converter::FromV8>::from_v8(
                            scope,
                            create_value,
                        ),
                    );
            }
            {
                let create_new_name = v8::String::new(scope, CREATE_NEW).unwrap();
                if true {
                    if !obj
                        .has_own_property(scope, create_new_name.into())
                        .unwrap_or(false)
                    {
                        ::core::panicking::panic(
                            "assertion failed: obj.has_own_property(scope, create_new_name.into()).unwrap_or(false)",
                        )
                    }
                }
                let create_new_value = obj.get(scope, create_new_name.into()).unwrap();
                builder
                    .create_new(
                        <bool as crate::js::converter::FromV8>::from_v8(
                            scope,
                            create_new_value,
                        ),
                    );
            }
            {
                let read_name = v8::String::new(scope, READ).unwrap();
                if true {
                    if !obj.has_own_property(scope, read_name.into()).unwrap_or(false) {
                        ::core::panicking::panic(
                            "assertion failed: obj.has_own_property(scope, read_name.into()).unwrap_or(false)",
                        )
                    }
                }
                let read_value = obj.get(scope, read_name.into()).unwrap();
                builder
                    .read(
                        <bool as crate::js::converter::FromV8>::from_v8(
                            scope,
                            read_value,
                        ),
                    );
            }
            {
                let truncate_name = v8::String::new(scope, TRUNCATE).unwrap();
                if true {
                    if !obj
                        .has_own_property(scope, truncate_name.into())
                        .unwrap_or(false)
                    {
                        ::core::panicking::panic(
                            "assertion failed: obj.has_own_property(scope, truncate_name.into()).unwrap_or(false)",
                        )
                    }
                }
                let truncate_value = obj.get(scope, truncate_name.into()).unwrap();
                builder
                    .truncate(
                        <bool as crate::js::converter::FromV8>::from_v8(
                            scope,
                            truncate_value,
                        ),
                    );
            }
            {
                let write_name = v8::String::new(scope, WRITE).unwrap();
                if true {
                    if !obj.has_own_property(scope, write_name.into()).unwrap_or(false) {
                        ::core::panicking::panic(
                            "assertion failed: obj.has_own_property(scope, write_name.into()).unwrap_or(false)",
                        )
                    }
                }
                let write_value = obj.get(scope, write_name.into()).unwrap();
                builder
                    .write(
                        <bool as crate::js::converter::FromV8>::from_v8(
                            scope,
                            write_value,
                        ),
                    );
            }
            builder.build().unwrap()
        }
    }
    pub fn fs_open(
        resource_table: ResourceTableArc,
        path: &Path,
        opts: FsOpenOptions,
    ) -> TheResult<ResourceId> {
        match std::fs::OpenOptions::new()
            .append(opts.append)
            .create(opts.create)
            .create_new(opts.create_new)
            .read(opts.read)
            .truncate(opts.truncate)
            .write(opts.write)
            .open(path)
        {
            Ok(file) => {
                let mut resource_table = (resource_table).lock().unwrap();
                Ok(resource_table.add_file(file))
            }
            Err(e) => {
                Err(
                    TheErr::OpenFileFailed(path.to_string_lossy().to_compact_string(), e),
                )
            }
        }
    }
    pub async fn async_fs_open(
        resource_table: ResourceTableArc,
        path: &Path,
        opts: FsOpenOptions,
    ) -> TheResult<ResourceId> {
        match tokio::fs::OpenOptions::new()
            .append(opts.append)
            .create(opts.create)
            .create_new(opts.create_new)
            .read(opts.read)
            .truncate(opts.truncate)
            .write(opts.write)
            .open(path)
            .await
        {
            Ok(file) => {
                let file = file.into_std().await;
                let mut resource_table = (resource_table).lock().unwrap();
                Ok(resource_table.add_file(file))
            }
            Err(e) => {
                Err(
                    TheErr::OpenFileFailed(path.to_string_lossy().to_compact_string(), e),
                )
            }
        }
    }
    pub struct FsOpenFuture {
        pub promise: v8::Global<v8::PromiseResolver>,
        pub maybe_result: Option<TheResult<Vec<u8>>>,
    }
    impl JsFuture for FsOpenFuture {
        fn run(&mut self, scope: &mut v8::PinScope) {
            {
                {
                    let lvl = ::log::Level::Trace;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!("|FsOpenFuture|"),
                            lvl,
                            &(
                                "rsvim_core::js::binding::global_rsvim::fs::open",
                                "rsvim_core::js::binding::global_rsvim::fs::open",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            let result = self.maybe_result.take().unwrap();
            if let Err(e) = result {
                let message = v8::String::new(scope, &e.to_string()).unwrap();
                let exception = v8::Exception::error(scope, message);
                binding::set_exception_code(scope, exception, &e);
                self.promise.open(scope).reject(scope, exception);
                return;
            }
            let result = result.unwrap();
            let file_rid = postcard::from_bytes::<ResourceId>(&result).unwrap();
            let file_rid = Into::<i32>::into(file_rid);
            let file_rid = file_rid.to_v8(scope);
            self.promise.open(scope).resolve(scope, file_rid).unwrap();
        }
    }
}
/// `Rsvim.fs.open` API.
pub fn open<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
    mut rv: v8::ReturnValue,
) {
    if true {
        if !(args.length() == 2) {
            ::core::panicking::panic("assertion failed: args.length() == 2")
        }
    }
    if true {
        if !(args.get(0).is_string() || args.get(0).is_string_object()) {
            ::core::panicking::panic("assertion failed: is_v8_str!(args.get(0))")
        }
    }
    let filename = args.get(0).to_rust_string_lossy(scope);
    if true {
        if !args.get(1).is_object() {
            ::core::panicking::panic("assertion failed: args.get(1).is_object()")
        }
    }
    let options = FsOpenOptions::from_v8(scope, args.get(1));
    {
        {
            let lvl = ::log::Level::Trace;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api::log(
                    { ::log::__private_api::GlobalLogger },
                    format_args!("Rsvim.fs.open:{0:?} {1:?}", filename, options),
                    lvl,
                    &(
                        "rsvim_core::js::binding::global_rsvim::fs",
                        "rsvim_core::js::binding::global_rsvim::fs",
                        ::log::__private_api::loc(),
                    ),
                    (),
                );
            }
        }
    };
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);
    let state_rc = JsRuntime::state(scope);
    let open_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();
        move |maybe_result: Option<TheResult<Vec<u8>>>| {
            let fut = FsOpenFuture {
                promise: promise.clone(),
                maybe_result,
            };
            let mut state = state_rc.borrow_mut();
            state.pending_futures.push(Box::new(fut));
        }
    };
    let mut state = state_rc.borrow_mut();
    let task_id = js::TaskId::next();
    let filename = Path::new(&filename);
    pending::create_fs_open(&mut state, task_id, filename, options, Box::new(open_cb));
    rv.set(promise.into());
}
