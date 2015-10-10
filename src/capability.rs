// Copyright (c) 2013-2015 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Hooks for for the RPC system.
//!
//! Roughly corresponds to capability.h in the C++ implementation.

use any_pointer;
use private::capability::{CallContextHook, ClientHook, RequestHook, ResponseHook};


pub struct RemotePromise<Results> where Results: ::traits::Pipelined {
    #[cfg(feature = "rpc")]
    pub answer_promise: ::gj::Promise<Box<ResponseHook>, ::Error>,
    pub pipeline: Results::Pipeline,
}

pub struct Request<Params, Results> {
    pub marker: ::std::marker::PhantomData<(Params, Results)>,
    pub hook: Box<RequestHook>
}

impl <Params, Results> Request <Params, Results> {
    pub fn new(hook: Box<RequestHook>) -> Request <Params, Results> {
        Request { hook: hook, marker: ::std::marker::PhantomData }
    }
}

#[cfg(feature = "rpc")]
impl <Params, Results> Request <Params, Results>
where Results: ::traits::Pipelined,
      <Results as ::traits::Pipelined>::Pipeline: FromTypelessPipeline
{
    pub fn send(self) -> RemotePromise<Results> {
        let RemotePromise {answer_promise, pipeline, ..} = self.hook.send();
        RemotePromise { answer_promise : answer_promise,
                        pipeline : FromTypelessPipeline::new(pipeline)
                      }
    }
}

pub struct CallContext<Params, Results> {
    pub marker: ::std::marker::PhantomData<(Params, Results)>,
    pub hook: Box<CallContextHook>,
}

impl <Params, Results> CallContext<Params, Results> {
    pub fn fail(self, message: String) {self.hook.fail(message);}
    pub fn done(self) {self.hook.done();}
}

impl <Params, Results> CallContext<Params, Results> {
    pub fn get<'a>(&'a mut self) -> (<Params as ::traits::Owned<'a>>::Reader,
                                     <Results as ::traits::Owned<'a>>::Builder)
        where Params: ::traits::Owned<'a>, Results: ::traits::Owned<'a>
    {
        let (any_params, any_results) = self.hook.get();
        (any_params.get_as().unwrap(), any_results.get_as().unwrap())
    }
}

pub trait FromTypelessPipeline {
    fn new (typeless : any_pointer::Pipeline) -> Self;
}

pub trait FromClientHook {
    fn new(Box<ClientHook>) -> Self;
}

pub trait Server {
    fn dispatch_call(&mut self, interface_id : u64, method_id : u16,
                     context : CallContext<any_pointer::Reader, any_pointer::Builder>);
}
