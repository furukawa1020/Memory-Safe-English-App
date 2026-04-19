import 'package:flutter/material.dart';

import '../../../app/app_scope.dart';
import '../../../core/api/api_exception.dart';

enum AuthMode { login, register }

class AuthScreen extends StatefulWidget {
  const AuthScreen({super.key});

  @override
  State<AuthScreen> createState() => _AuthScreenState();
}

class _AuthScreenState extends State<AuthScreen> {
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();
  final _displayNameController = TextEditingController();
  final _formKey = GlobalKey<FormState>();
  AuthMode _mode = AuthMode.login;
  String? _errorText;

  @override
  void dispose() {
    _emailController.dispose();
    _passwordController.dispose();
    _displayNameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final sessionController = AppScope.of(context).sessionController;

    return Scaffold(
      body: Container(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            colors: [Color(0xFFE8F2ED), Color(0xFFF6EBDD)],
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
          ),
        ),
        child: SafeArea(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 460),
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: AnimatedBuilder(
                      animation: sessionController,
                      builder: (context, _) {
                        return Form(
                          key: _formKey,
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                'Memory-Safe English',
                                style: Theme.of(context).textTheme.headlineMedium?.copyWith(fontWeight: FontWeight.w700),
                              ),
                              const SizedBox(height: 8),
                              Text(
                                '保持しなくても読める・聞ける・話せる感覚をつくる学習 UI',
                                style: Theme.of(context).textTheme.bodyMedium,
                              ),
                              const SizedBox(height: 20),
                              SegmentedButton<AuthMode>(
                                segments: const [
                                  ButtonSegment(value: AuthMode.login, label: Text('Login')),
                                  ButtonSegment(value: AuthMode.register, label: Text('Register')),
                                ],
                                selected: {_mode},
                                onSelectionChanged: (selection) {
                                  setState(() {
                                    _mode = selection.first;
                                    _errorText = null;
                                  });
                                },
                              ),
                              const SizedBox(height: 20),
                              if (_mode == AuthMode.register) ...[
                                TextFormField(
                                  controller: _displayNameController,
                                  decoration: const InputDecoration(labelText: 'Display name'),
                                  validator: (value) {
                                    if (_mode == AuthMode.register && (value == null || value.trim().isEmpty)) {
                                      return '表示名を入力してください';
                                    }
                                    return null;
                                  },
                                ),
                                const SizedBox(height: 12),
                              ],
                              TextFormField(
                                controller: _emailController,
                                decoration: const InputDecoration(labelText: 'Email'),
                                keyboardType: TextInputType.emailAddress,
                                validator: (value) {
                                  if (value == null || !value.contains('@')) {
                                    return 'メールアドレスを入力してください';
                                  }
                                  return null;
                                },
                              ),
                              const SizedBox(height: 12),
                              TextFormField(
                                controller: _passwordController,
                                decoration: const InputDecoration(labelText: 'Password'),
                                obscureText: true,
                                validator: (value) {
                                  if (value == null || value.length < 12) {
                                    return '12文字以上のパスワードを入力してください';
                                  }
                                  return null;
                                },
                              ),
                              if (_errorText != null) ...[
                                const SizedBox(height: 12),
                                Text(_errorText!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
                              ],
                              const SizedBox(height: 18),
                              SizedBox(
                                width: double.infinity,
                                child: FilledButton(
                                  onPressed: sessionController.isBusy ? null : () => _submit(context),
                                  child: Padding(
                                    padding: const EdgeInsets.symmetric(vertical: 12),
                                    child: Text(
                                      sessionController.isBusy
                                          ? 'Connecting...'
                                          : _mode == AuthMode.login
                                              ? 'Login'
                                              : 'Create account',
                                    ),
                                  ),
                                ),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _submit(BuildContext context) async {
    if (!_formKey.currentState!.validate()) {
      return;
    }

    final scope = AppScope.of(context);
    try {
      if (_mode == AuthMode.login) {
        await scope.sessionController.login(
          repository: scope.authRepository,
          email: _emailController.text.trim(),
          password: _passwordController.text,
        );
      } else {
        await scope.sessionController.register(
          repository: scope.authRepository,
          email: _emailController.text.trim(),
          password: _passwordController.text,
          displayName: _displayNameController.text.trim(),
        );
      }
      if (!mounted) {
        return;
      }
      setState(() => _errorText = null);
    } on ApiException catch (error) {
      setState(() => _errorText = error.message);
    } catch (_) {
      setState(() => _errorText = '接続に失敗しました');
    }
  }
}
