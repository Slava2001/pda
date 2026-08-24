#[macro_export]
macro_rules! create_imc_interface {
    (
        $vis:vis interface $name:ident {
            $(
                fn $method:ident(
                    $( $arg:ident : $arg_ty:ty ),* $(,)?
                )
                $(-> $ret:ty)?
                ;
            )*
        }
    ) => {
        paste::paste! {
            $vis struct $name {
                $(
                    [<$method _channel>]:
                        tokio::sync::mpsc::Sender<
                            create_imc_interface! {
                                @channel_type
                                args: {$($arg_ty),*}
                                ret: {$($ret)?}
                            }
                        >,
                )*
            }

            impl Clone for $name {
                fn clone(&self) -> Self {
                    Self {
                        $(
                            [<$method _channel>]:
                                self.[<$method _channel>].clone(),
                        )*
                    }
                }
            }

            impl $name {
                $(
                    create_imc_interface! {
                        @interface_method
                        $method
                        args: {$($arg : $arg_ty),*}
                        ret: {$($ret)?}
                    }
                )*
            }

            #[allow(non_camel_case_types)]
            $vis enum [<$name Event>] {
                $(
                    $method(create_imc_interface! {
                                @channel_type
                                args: {$($arg_ty),*}
                                ret: {$($ret)?}
                            })
                ),*
            }

            $vis struct [<$name BackendBuilder>] {
                $(
                    [<$method _cb>]:
                        Option<
                            Box<
                                dyn FnMut(
                                    $($arg_ty),*
                                )
                                -> std::pin::Pin<
                                    Box<
                                        dyn Future<
                                            Output =
                                                create_imc_interface! {
                                                    @callback_type
                                                    ret: {$($ret)?}
                                                }
                                        > + Send
                                    >
                                >
                                + Send
                            >
                        >,
                )*
            }

            impl [<$name BackendBuilder>] {
                $(
                    pub fn [<on_ $method>]<F, Fut>(
                        mut self,
                        mut callback: F
                    ) -> Self
                    where
                        F: FnMut($($arg_ty),*) -> Fut
                            + Send
                            + 'static,

                        Fut: Future<
                                Output =
                                    create_imc_interface! {
                                        @callback_type
                                        ret: {$($ret)?}
                                    }
                            >
                            + Send
                            + 'static,
                    {
                        self.[<$method _cb>] =
                            Some(Box::new(
                                move |$($arg),*| {
                                    Box::pin(
                                        callback($($arg),*)
                                    )
                                }
                            ));

                        self
                    }
                )*

                fn build(
                    self
                ) -> anyhow::Result<[<$name Backend>]> {

                    $(
                        let ([<$method _tx>], [<$method _rx>]) =
                            tokio::sync::mpsc::channel(10);
                    )*

                    Ok(
                        [<$name Backend>] {
                            self_if: $name {
                                $(
                                    [<$method _channel>]:
                                        [<$method _tx>],
                                )*
                            },

                            $(
                                [<$method _channel>]: [<$method _rx>],
                                [<$method _cb>]: self.[<$method _cb>],
                            )*
                        }
                    )
                }
            }

            struct [<$name Backend>] {
                self_if: $name,

                $(
                    [<$method _channel>]:
                        tokio::sync::mpsc::Receiver<
                            create_imc_interface! {
                                @channel_type
                                args: {$($arg_ty),*}
                                ret: {$($ret)?}
                            }
                        >,

                    [<$method _cb>]:
                        Option<Box<
                            dyn FnMut(
                                $($arg_ty),*
                            )
                            -> std::pin::Pin<
                                Box<
                                    dyn Future<
                                        Output =
                                            create_imc_interface! {
                                                @callback_type
                                                ret: {$($ret)?}
                                            }
                                    > + Send
                                >
                            >
                            + Send
                        >>,
                )*
            }

            impl [<$name Backend>] {
                pub fn builder() -> [<$name BackendBuilder>] {
                    [<$name BackendBuilder>] {
                        $(
                            [<$method _cb>]: None,
                        )*
                    }
                }

                pub fn get_if(&self) -> $name {
                    self.self_if.clone()
                }

                pub async fn poll(&mut self) {
                    tokio::select! {
                        $(
                            Some((($($arg,)*), rc)) = self.[<$method _channel>].recv() => {
                                if let Some(cb) = self.[<$method _cb>].as_mut() {
                                    let result = cb($($arg),*).await;
                                    let _ = rc.send(result);
                                }
                            }
                        )*
                    }
                }

                pub async fn poll_event(&mut self) -> [<$name Event>] {
                    tokio::select! {
                        $(
                            Some(args) = self.[<$method _channel>].recv() => {
                                return [<$name Event>]::$method(args);
                            }
                        )*
                    }
                }
            }
        }
    };

    (
        @interface_method
        $method:ident
        args: {$($arg:ident : $arg_ty:ty),*}
        ret: {$($ret:ty)?}
    ) => {
        paste::paste! {
            pub async fn $method(
                &mut self,
                $($arg: $arg_ty),*
            ) -> anyhow::Result<
                    create_imc_interface! {
                        @callback_type
                        ret: {$($ret)?}
                    }
                >
            {
                use anyhow::Context;

                let (rc_tx, rc_rx) =
                    tokio::sync::oneshot::channel();

                self.[<$method _channel>]
                    .send((
                        ($($arg,)*),
                        rc_tx
                    ))
                    .await
                    .context("Failed to send request")?;

                rc_rx
                    .await
                    .context("Failed to receive response")
            }
        }
    };

    (
        @channel_type
        args: {$($arg_ty:ty),*}
        ret: {$($ret:ty)?}
    ) => {
        (
            ($($arg_ty,)*),
            tokio::sync::oneshot::Sender<
                create_imc_interface! {
                    @callback_type
                    ret: {$($ret)?}
                }
            >
        )
    };

    (
        @callback_type
        ret: {}
    ) => {
        ()
    };

    (
        @callback_type
        ret: {$ret:ty}
    ) => {
        $ret
    };
}
